//! # Structured Query Language
//!
//! This module provides an internal representation of a structured query language
//! for building composable query expressions with filtering, comparison, and logical operations.
//!
//! The structured query system uses the visitor pattern to allow different backends
//! to translate queries into their native query languages.

use serde::{Deserialize, Serialize};

/// Defines interface for IR translation using a visitor pattern.
pub trait Visitor {
    /// The result type of visiting expressions
    type Output;

    /// Allowed comparators for the visitor
    fn allowed_comparators(&self) -> Option<&[Comparator]> {
        None
    }

    /// Allowed operators for the visitor
    fn allowed_operators(&self) -> Option<&[Operator]> {
        None
    }

    /// Validate that a function (operator or comparator) is allowed
    fn validate_func(&self, func: &dyn Function) -> Result<(), String> {
        if let Some(allowed_operators) = self.allowed_operators() {
            if let Some(op) = func.as_operator() {
                if !allowed_operators.contains(&op) {
                    return Err(format!(
                        "Received disallowed operator {op:?}. Allowed operators are {allowed_operators:?}"
                    ));
                }
            }
        }

        if let Some(allowed_comparators) = self.allowed_comparators() {
            if let Some(comp) = func.as_comparator() {
                if !allowed_comparators.contains(&comp) {
                    return Err(format!(
                        "Received disallowed comparator {comp:?}. Allowed comparators are {allowed_comparators:?}"
                    ));
                }
            }
        }

        Ok(())
    }

    /// Translate an Operation
    fn visit_operation(&self, operation: &Operation) -> Result<Self::Output, String>;

    /// Translate a Comparison
    fn visit_comparison(&self, comparison: &Comparison) -> Result<Self::Output, String>;

    /// Translate a StructuredQuery
    fn visit_structured_query(
        &self,
        structured_query: &StructuredQuery,
    ) -> Result<Self::Output, String>;
}

/// Trait for functions that can be validated
pub trait Function {
    /// Get as operator if this is an operator
    fn as_operator(&self) -> Option<Operator>;
    /// Get as comparator if this is a comparator
    fn as_comparator(&self) -> Option<Comparator>;
}

impl Function for Operator {
    fn as_operator(&self) -> Option<Operator> {
        Some(*self)
    }

    fn as_comparator(&self) -> Option<Comparator> {
        None
    }
}

impl Function for Comparator {
    fn as_operator(&self) -> Option<Operator> {
        None
    }

    fn as_comparator(&self) -> Option<Comparator> {
        Some(*self)
    }
}

/// Base trait for all expressions
pub trait Expr {
    /// Accept a visitor and return the result
    fn accept<V: Visitor>(&self, visitor: &V) -> Result<V::Output, String>;
}

/// Enumerator of the logical operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Operator {
    /// Logical AND operation
    And,
    /// Logical OR operation
    Or,
    /// Logical NOT operation
    Not,
}

impl std::fmt::Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operator::And => write!(f, "and"),
            Operator::Or => write!(f, "or"),
            Operator::Not => write!(f, "not"),
        }
    }
}

/// Enumerator of the comparison operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Comparator {
    /// Equal to
    Eq,
    /// Not equal to
    Ne,
    /// Greater than
    Gt,
    /// Greater than or equal to
    Gte,
    /// Less than
    Lt,
    /// Less than or equal to
    Lte,
    /// Contains
    Contain,
    /// Like (pattern matching)
    Like,
    /// In (membership)
    In,
    /// Not in (not membership)
    Nin,
}

impl std::fmt::Display for Comparator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Comparator::Eq => write!(f, "eq"),
            Comparator::Ne => write!(f, "ne"),
            Comparator::Gt => write!(f, "gt"),
            Comparator::Gte => write!(f, "gte"),
            Comparator::Lt => write!(f, "lt"),
            Comparator::Lte => write!(f, "lte"),
            Comparator::Contain => write!(f, "contain"),
            Comparator::Like => write!(f, "like"),
            Comparator::In => write!(f, "in"),
            Comparator::Nin => write!(f, "nin"),
        }
    }
}

/// Enum representing all possible filter directives
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FilterDirective {
    /// A comparison operation
    Comparison(Comparison),
    /// A logical operation
    Operation(Operation),
}

impl FilterDirective {
    /// Create a comparison filter directive
    pub fn comparison(comparator: Comparator, attribute: String, value: serde_json::Value) -> Self {
        Self::Comparison(Comparison::new(comparator, attribute, value))
    }

    /// Create an operation filter directive
    pub fn operation(operator: Operator, arguments: Vec<FilterDirective>) -> Self {
        Self::Operation(Operation::new(operator, arguments))
    }
}

impl Expr for FilterDirective {
    fn accept<V: Visitor>(&self, visitor: &V) -> Result<V::Output, String> {
        match self {
            FilterDirective::Comparison(comp) => comp.accept(visitor),
            FilterDirective::Operation(op) => op.accept(visitor),
        }
    }
}

/// Comparison to a value
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Comparison {
    /// The comparator to use
    pub comparator: Comparator,
    /// The attribute to compare
    pub attribute: String,
    /// The value to compare to
    pub value: serde_json::Value,
}

impl Comparison {
    /// Create a new Comparison
    ///
    /// # Arguments
    ///
    /// * `comparator` - The comparator to use
    /// * `attribute` - The attribute to compare
    /// * `value` - The value to compare to
    pub fn new(comparator: Comparator, attribute: String, value: serde_json::Value) -> Self {
        Self {
            comparator,
            attribute,
            value,
        }
    }
}

impl Expr for Comparison {
    fn accept<V: Visitor>(&self, visitor: &V) -> Result<V::Output, String> {
        visitor.validate_func(&self.comparator)?;
        visitor.visit_comparison(self)
    }
}

/// Logical operation over other directives
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Operation {
    /// The operator to use
    pub operator: Operator,
    /// The arguments to the operator
    pub arguments: Vec<FilterDirective>,
}

impl Operation {
    /// Create a new Operation
    ///
    /// # Arguments
    ///
    /// * `operator` - The operator to use
    /// * `arguments` - The arguments to the operator
    pub fn new(operator: Operator, arguments: Vec<FilterDirective>) -> Self {
        Self {
            operator,
            arguments,
        }
    }
}

impl Expr for Operation {
    fn accept<V: Visitor>(&self, visitor: &V) -> Result<V::Output, String> {
        visitor.validate_func(&self.operator)?;
        visitor.visit_operation(self)
    }
}

/// Structured query with optional filtering and limiting
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructuredQuery {
    /// Query string
    pub query: String,
    /// Optional filtering expression
    pub filter: Option<FilterDirective>,
    /// Optional limit on the number of results
    pub limit: Option<u32>,
}

impl StructuredQuery {
    /// Create a new StructuredQuery
    ///
    /// # Arguments
    ///
    /// * `query` - The query string
    /// * `filter` - Optional filtering expression
    /// * `limit` - Optional limit on the number of results
    pub fn new(query: String, filter: Option<FilterDirective>, limit: Option<u32>) -> Self {
        Self {
            query,
            filter,
            limit,
        }
    }

    /// Create a simple query without filters or limits
    ///
    /// # Arguments
    ///
    /// * `query` - The query string
    pub fn simple(query: String) -> Self {
        Self {
            query,
            filter: None,
            limit: None,
        }
    }
}

impl Expr for StructuredQuery {
    fn accept<V: Visitor>(&self, visitor: &V) -> Result<V::Output, String> {
        visitor.visit_structured_query(self)
    }
}

/// Helper functions for creating common query expressions
pub mod builders {
    use super::*;

    /// Create an equality comparison
    ///
    /// # Arguments
    ///
    /// * `attribute` - The attribute to compare
    /// * `value` - The value to compare to
    pub fn eq(attribute: &str, value: serde_json::Value) -> FilterDirective {
        FilterDirective::comparison(Comparator::Eq, attribute.to_string(), value)
    }

    /// Create a not-equal comparison
    ///
    /// # Arguments
    ///
    /// * `attribute` - The attribute to compare
    /// * `value` - The value to compare to
    pub fn ne(attribute: &str, value: serde_json::Value) -> FilterDirective {
        FilterDirective::comparison(Comparator::Ne, attribute.to_string(), value)
    }

    /// Create a greater-than comparison
    ///
    /// # Arguments
    ///
    /// * `attribute` - The attribute to compare
    /// * `value` - The value to compare to
    pub fn gt(attribute: &str, value: serde_json::Value) -> FilterDirective {
        FilterDirective::comparison(Comparator::Gt, attribute.to_string(), value)
    }

    /// Create a greater-than-or-equal comparison
    ///
    /// # Arguments
    ///
    /// * `attribute` - The attribute to compare
    /// * `value` - The value to compare to
    pub fn gte(attribute: &str, value: serde_json::Value) -> FilterDirective {
        FilterDirective::comparison(Comparator::Gte, attribute.to_string(), value)
    }

    /// Create a less-than comparison
    ///
    /// # Arguments
    ///
    /// * `attribute` - The attribute to compare
    /// * `value` - The value to compare to
    pub fn lt(attribute: &str, value: serde_json::Value) -> FilterDirective {
        FilterDirective::comparison(Comparator::Lt, attribute.to_string(), value)
    }

    /// Create a less-than-or-equal comparison
    ///
    /// # Arguments
    ///
    /// * `attribute` - The attribute to compare
    /// * `value` - The value to compare to
    pub fn lte(attribute: &str, value: serde_json::Value) -> FilterDirective {
        FilterDirective::comparison(Comparator::Lte, attribute.to_string(), value)
    }

    /// Create a contains comparison
    ///
    /// # Arguments
    ///
    /// * `attribute` - The attribute to compare
    /// * `value` - The value to compare to
    pub fn contain(attribute: &str, value: serde_json::Value) -> FilterDirective {
        FilterDirective::comparison(Comparator::Contain, attribute.to_string(), value)
    }

    /// Create a like comparison
    ///
    /// # Arguments
    ///
    /// * `attribute` - The attribute to compare
    /// * `value` - The value to compare to
    pub fn like(attribute: &str, value: serde_json::Value) -> FilterDirective {
        FilterDirective::comparison(Comparator::Like, attribute.to_string(), value)
    }

    /// Create an in comparison
    ///
    /// # Arguments
    ///
    /// * `attribute` - The attribute to compare
    /// * `value` - The value to compare to
    pub fn r#in(attribute: &str, value: serde_json::Value) -> FilterDirective {
        FilterDirective::comparison(Comparator::In, attribute.to_string(), value)
    }

    /// Create a not-in comparison
    ///
    /// # Arguments
    ///
    /// * `attribute` - The attribute to compare
    /// * `value` - The value to compare to
    pub fn nin(attribute: &str, value: serde_json::Value) -> FilterDirective {
        FilterDirective::comparison(Comparator::Nin, attribute.to_string(), value)
    }

    /// Create an AND operation
    ///
    /// # Arguments
    ///
    /// * `arguments` - The arguments to the AND operation
    pub fn and(arguments: Vec<FilterDirective>) -> FilterDirective {
        FilterDirective::operation(Operator::And, arguments)
    }

    /// Create an OR operation
    ///
    /// # Arguments
    ///
    /// * `arguments` - The arguments to the OR operation
    pub fn or(arguments: Vec<FilterDirective>) -> FilterDirective {
        FilterDirective::operation(Operator::Or, arguments)
    }

    /// Create a NOT operation
    ///
    /// # Arguments
    ///
    /// * `argument` - The argument to the NOT operation
    pub fn not(argument: FilterDirective) -> FilterDirective {
        FilterDirective::operation(Operator::Not, vec![argument])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Mock visitor for testing
    struct MockVisitor {
        allowed_comparators: Option<Vec<Comparator>>,
        allowed_operators: Option<Vec<Operator>>,
    }

    impl MockVisitor {
        fn new() -> Self {
            Self {
                allowed_comparators: None,
                allowed_operators: None,
            }
        }

        fn with_allowed_comparators(comparators: Vec<Comparator>) -> Self {
            Self {
                allowed_comparators: Some(comparators),
                allowed_operators: None,
            }
        }
    }

    impl Visitor for MockVisitor {
        type Output = String;

        fn allowed_comparators(&self) -> Option<&[Comparator]> {
            self.allowed_comparators.as_deref()
        }

        fn allowed_operators(&self) -> Option<&[Operator]> {
            self.allowed_operators.as_deref()
        }

        fn visit_operation(&self, operation: &Operation) -> Result<Self::Output, String> {
            Ok(format!(
                "Operation({:?}, {} args)",
                operation.operator,
                operation.arguments.len()
            ))
        }

        fn visit_comparison(&self, comparison: &Comparison) -> Result<Self::Output, String> {
            Ok(format!(
                "Comparison({:?}, {}, {})",
                comparison.comparator, comparison.attribute, comparison.value
            ))
        }

        fn visit_structured_query(
            &self,
            structured_query: &StructuredQuery,
        ) -> Result<Self::Output, String> {
            let filter_str = if let Some(_) = &structured_query.filter {
                "with filter"
            } else {
                "no filter"
            };
            let limit_str = if let Some(limit) = structured_query.limit {
                format!(", limit: {}", limit)
            } else {
                String::new()
            };
            Ok(format!(
                "Query(\"{}\", {}, {})",
                structured_query.query, filter_str, limit_str
            ))
        }
    }

    #[test]
    fn test_comparison_creation() {
        let comp = Comparison::new(Comparator::Eq, "name".to_string(), json!("John"));
        assert_eq!(comp.comparator, Comparator::Eq);
        assert_eq!(comp.attribute, "name");
        assert_eq!(comp.value, json!("John"));
    }

    #[test]
    fn test_filter_directive_creation() {
        let comp = FilterDirective::comparison(Comparator::Eq, "name".to_string(), json!("John"));
        match comp {
            FilterDirective::Comparison(c) => {
                assert_eq!(c.comparator, Comparator::Eq);
                assert_eq!(c.attribute, "name");
            }
            _ => panic!("Expected Comparison variant"),
        }
    }

    #[test]
    fn test_operation_creation() {
        let comp1 = FilterDirective::comparison(Comparator::Eq, "name".to_string(), json!("John"));
        let comp2 = FilterDirective::comparison(Comparator::Gt, "age".to_string(), json!(18));
        let op = FilterDirective::operation(Operator::And, vec![comp1, comp2]);
        match op {
            FilterDirective::Operation(o) => {
                assert_eq!(o.operator, Operator::And);
                assert_eq!(o.arguments.len(), 2);
            }
            _ => panic!("Expected Operation variant"),
        }
    }

    #[test]
    fn test_structured_query_creation() {
        let query = StructuredQuery::simple("test query".to_string());
        assert_eq!(query.query, "test query");
        assert!(query.filter.is_none());
        assert!(query.limit.is_none());
    }

    #[test]
    fn test_visitor_acceptance() {
        let visitor = MockVisitor::new();
        let comp = FilterDirective::comparison(Comparator::Eq, "name".to_string(), json!("John"));
        let result = comp.accept(&visitor).unwrap();
        assert!(result.contains("Comparison"));
        assert!(result.contains("Eq"));
        assert!(result.contains("name"));
    }

    #[test]
    fn test_visitor_validation_success() {
        let visitor = MockVisitor::with_allowed_comparators(vec![Comparator::Eq, Comparator::Gt]);
        let comp = FilterDirective::comparison(Comparator::Eq, "name".to_string(), json!("John"));
        let result = comp.accept(&visitor);
        assert!(result.is_ok());
    }

    #[test]
    fn test_visitor_validation_failure() {
        let visitor = MockVisitor::with_allowed_comparators(vec![Comparator::Eq, Comparator::Gt]);
        let comp = FilterDirective::comparison(Comparator::Lt, "age".to_string(), json!(18));
        let result = comp.accept(&visitor);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("disallowed comparator"));
    }

    #[test]
    fn test_builders() {
        let eq_comp = builders::eq("name", json!("John"));
        match &eq_comp {
            FilterDirective::Comparison(c) => {
                assert_eq!(c.comparator, Comparator::Eq);
                assert_eq!(c.attribute, "name");
            }
            _ => panic!("Expected Comparison variant"),
        }

        let gt_comp = builders::gt("age", json!(18));
        match &gt_comp {
            FilterDirective::Comparison(c) => {
                assert_eq!(c.comparator, Comparator::Gt);
                assert_eq!(c.attribute, "age");
            }
            _ => panic!("Expected Comparison variant"),
        }

        let and_op = builders::and(vec![eq_comp, gt_comp]);
        match and_op {
            FilterDirective::Operation(o) => {
                assert_eq!(o.operator, Operator::And);
                assert_eq!(o.arguments.len(), 2);
            }
            _ => panic!("Expected Operation variant"),
        }
    }

    #[test]
    fn test_serialization() {
        let comp = FilterDirective::comparison(Comparator::Eq, "name".to_string(), json!("John"));
        let serialized = serde_json::to_string(&comp).unwrap();
        let deserialized: FilterDirective = serde_json::from_str(&serialized).unwrap();
        assert_eq!(comp, deserialized);
    }

    #[test]
    fn test_display_traits() {
        assert_eq!(Operator::And.to_string(), "and");
        assert_eq!(Operator::Or.to_string(), "or");
        assert_eq!(Operator::Not.to_string(), "not");

        assert_eq!(Comparator::Eq.to_string(), "eq");
        assert_eq!(Comparator::Gt.to_string(), "gt");
        assert_eq!(Comparator::Contain.to_string(), "contain");
    }
}
