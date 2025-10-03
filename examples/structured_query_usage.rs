//! # Structured Query Usage Example
//!
//! This example demonstrates how to use the structured query system
//! to build complex queries with filtering, comparison, and logical operations.

use ferriclink_core::structured_query::{
    builders, Comparator, Expr, FilterDirective, Operator, StructuredQuery, Visitor,
};
use serde_json::json;

/// A simple visitor that converts structured queries to SQL-like strings
struct SqlVisitor;

impl Visitor for SqlVisitor {
    type Output = String;

    fn visit_operation(&self, operation: &ferriclink_core::structured_query::Operation) -> Result<Self::Output, String> {
        match operation.operator {
            Operator::And => {
                let conditions: Result<Vec<String>, String> = operation
                    .arguments
                    .iter()
                    .map(|arg| arg.accept(self))
                    .collect();
                Ok(format!("({})", conditions?.join(" AND ")))
            }
            Operator::Or => {
                let conditions: Result<Vec<String>, String> = operation
                    .arguments
                    .iter()
                    .map(|arg| arg.accept(self))
                    .collect();
                Ok(format!("({})", conditions?.join(" OR ")))
            }
            Operator::Not => {
                if operation.arguments.len() != 1 {
                    return Err("NOT operation must have exactly one argument".to_string());
                }
                let condition = operation.arguments[0].accept(self)?;
                Ok(format!("NOT ({})", condition))
            }
        }
    }

    fn visit_comparison(&self, comparison: &ferriclink_core::structured_query::Comparison) -> Result<Self::Output, String> {
        let value = match &comparison.value {
            serde_json::Value::String(s) => format!("'{}'", s),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            _ => format!("'{}'", comparison.value),
        };

        match comparison.comparator {
            Comparator::Eq => Ok(format!("{} = {}", comparison.attribute, value)),
            Comparator::Ne => Ok(format!("{} != {}", comparison.attribute, value)),
            Comparator::Gt => Ok(format!("{} > {}", comparison.attribute, value)),
            Comparator::Gte => Ok(format!("{} >= {}", comparison.attribute, value)),
            Comparator::Lt => Ok(format!("{} < {}", comparison.attribute, value)),
            Comparator::Lte => Ok(format!("{} <= {}", comparison.attribute, value)),
            Comparator::Contain => Ok(format!("{} LIKE '%{}%'", comparison.attribute, value.trim_matches('\''))),
            Comparator::Like => Ok(format!("{} LIKE {}", comparison.attribute, value)),
            Comparator::In => Ok(format!("{} IN ({})", comparison.attribute, value)),
            Comparator::Nin => Ok(format!("{} NOT IN ({})", comparison.attribute, value)),
        }
    }

    fn visit_structured_query(&self, structured_query: &StructuredQuery) -> Result<Self::Output, String> {
        let mut sql = format!("SELECT * FROM documents WHERE content LIKE '%{}%'", structured_query.query);
        
        if let Some(filter) = &structured_query.filter {
            let filter_sql = filter.accept(self)?;
            sql = format!("SELECT * FROM documents WHERE content LIKE '%{}%' AND {}", structured_query.query, filter_sql);
        }

        if let Some(limit) = structured_query.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        Ok(sql)
    }
}

/// A visitor that converts structured queries to MongoDB-like queries
struct MongoVisitor;

impl Visitor for MongoVisitor {
    type Output = serde_json::Value;

    fn visit_operation(&self, operation: &ferriclink_core::structured_query::Operation) -> Result<Self::Output, String> {
        match operation.operator {
            Operator::And => {
                let conditions: Result<Vec<serde_json::Value>, String> = operation
                    .arguments
                    .iter()
                    .map(|arg| arg.accept(self))
                    .collect();
                Ok(json!({ "$and": conditions? }))
            }
            Operator::Or => {
                let conditions: Result<Vec<serde_json::Value>, String> = operation
                    .arguments
                    .iter()
                    .map(|arg| arg.accept(self))
                    .collect();
                Ok(json!({ "$or": conditions? }))
            }
            Operator::Not => {
                if operation.arguments.len() != 1 {
                    return Err("NOT operation must have exactly one argument".to_string());
                }
                let condition = operation.arguments[0].accept(self)?;
                Ok(json!({ "$not": condition }))
            }
        }
    }

    fn visit_comparison(&self, comparison: &ferriclink_core::structured_query::Comparison) -> Result<Self::Output, String> {
        let field = &comparison.attribute;
        let value = &comparison.value;

        match comparison.comparator {
            Comparator::Eq => Ok(json!({ field: value })),
            Comparator::Ne => Ok(json!({ field: { "$ne": value } })),
            Comparator::Gt => Ok(json!({ field: { "$gt": value } })),
            Comparator::Gte => Ok(json!({ field: { "$gte": value } })),
            Comparator::Lt => Ok(json!({ field: { "$lt": value } })),
            Comparator::Lte => Ok(json!({ field: { "$lte": value } })),
            Comparator::Contain => Ok(json!({ field: { "$regex": format!(".*{}.*", value.as_str().unwrap_or("")), "$options": "i" } })),
            Comparator::Like => Ok(json!({ field: { "$regex": value.as_str().unwrap_or(""), "$options": "i" } })),
            Comparator::In => Ok(json!({ field: { "$in": value } })),
            Comparator::Nin => Ok(json!({ field: { "$nin": value } })),
        }
    }

    fn visit_structured_query(&self, structured_query: &StructuredQuery) -> Result<Self::Output, String> {
        let mut query = json!({
            "content": { "$regex": format!(".*{}.*", structured_query.query), "$options": "i" }
        });

        if let Some(filter) = &structured_query.filter {
            let filter_condition = filter.accept(self)?;
            query = json!({
                "$and": [
                    { "content": { "$regex": format!(".*{}.*", structured_query.query), "$options": "i" } },
                    filter_condition
                ]
            });
        }

        let mut result = json!({ "filter": query });
        if let Some(limit) = structured_query.limit {
            result["limit"] = json!(limit);
        }

        Ok(result)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== FerricLink Structured Query Example ===\n");

    // Example 1: Simple query
    println!("1. Simple Query:");
    let simple_query = StructuredQuery::simple("machine learning".to_string());
    println!("Query: {:?}\n", simple_query);

    // Example 2: Query with filters using builders
    println!("2. Query with Filters:");
    let age_filter = builders::gte("age", json!(18));
    let category_filter = builders::eq("category", json!("technology"));
    let and_filter = builders::and(vec![age_filter, category_filter]);

    let filtered_query = StructuredQuery::new(
        "artificial intelligence".to_string(),
        Some(and_filter),
        Some(10),
    );
    println!("Query: {:?}\n", filtered_query);

    // Example 3: Complex query with nested operations
    println!("3. Complex Query with Nested Operations:");
    let name_filter = builders::like("name", json!("John%"));
    let age_filter = builders::gte("age", json!(25));
    let location_filter = builders::r#in("location", json!(["New York", "San Francisco"]));
    
    let or_condition = builders::or(vec![age_filter, location_filter]);
    
    let complex_filter = builders::and(vec![name_filter, or_condition]);

    let complex_query = StructuredQuery::new(
        "data science".to_string(),
        Some(complex_filter),
        Some(5),
    );
    println!("Query: {:?}\n", complex_query);

    // Example 4: Convert to SQL
    println!("4. Converting to SQL:");
    let sql_visitor = SqlVisitor;
    
    let sql_simple = simple_query.accept(&sql_visitor)?;
    println!("Simple SQL: {}", sql_simple);

    let sql_filtered = filtered_query.accept(&sql_visitor)?;
    println!("Filtered SQL: {}", sql_filtered);

    let sql_complex = complex_query.accept(&sql_visitor)?;
    println!("Complex SQL: {}", sql_complex);
    println!();

    // Example 5: Convert to MongoDB
    println!("5. Converting to MongoDB:");
    let mongo_visitor = MongoVisitor;
    
    let mongo_simple = simple_query.accept(&mongo_visitor)?;
    println!("Simple MongoDB: {}", serde_json::to_string_pretty(&mongo_simple)?);

    let mongo_filtered = filtered_query.accept(&mongo_visitor)?;
    println!("Filtered MongoDB: {}", serde_json::to_string_pretty(&mongo_filtered)?);

    let mongo_complex = complex_query.accept(&mongo_visitor)?;
    println!("Complex MongoDB: {}", serde_json::to_string_pretty(&mongo_complex)?);
    println!();

    // Example 6: Validation with restricted visitors
    println!("6. Validation with Restricted Visitors:");
    
    struct RestrictedVisitor {
        allowed_comparators: Vec<Comparator>,
        allowed_operators: Vec<Operator>,
    }

    impl Visitor for RestrictedVisitor {
        type Output = String;

        fn allowed_comparators(&self) -> Option<&[Comparator]> {
            Some(&self.allowed_comparators)
        }

        fn allowed_operators(&self) -> Option<&[Operator]> {
            Some(&self.allowed_operators)
        }

        fn visit_operation(&self, operation: &Operation) -> Result<Self::Output, String> {
            Ok(format!("Operation({:?})", operation.operator))
        }

        fn visit_comparison(&self, comparison: &Comparison) -> Result<Self::Output, String> {
            Ok(format!("Comparison({:?})", comparison.comparator))
        }

        fn visit_structured_query(&self, structured_query: &StructuredQuery) -> Result<Self::Output, String> {
            Ok(format!("Query(\"{}\")", structured_query.query))
        }
    }

    let restricted_visitor = RestrictedVisitor {
        allowed_comparators: vec![Comparator::Eq, Comparator::Ne],
        allowed_operators: vec![Operator::And],
    };

    // This should work
    let valid_comparison = builders::eq("status", json!("active"));
    let valid_result = valid_comparison.accept(&restricted_visitor);
    println!("Valid comparison result: {:?}", valid_result);

    // This should fail
    let invalid_comparison = builders::gt("age", json!(18));
    let invalid_result = invalid_comparison.accept(&restricted_visitor);
    println!("Invalid comparison result: {:?}", invalid_result);

    println!("\n=== Example Complete ===");
    Ok(())
}
