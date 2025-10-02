//! Utility functions for FerricLink Core
//!
//! This module provides shared utility functions used across the FerricLink ecosystem.

/// Color codes for terminal output
pub mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const BLUE: &str = "\x1b[36;1m";
    pub const YELLOW: &str = "\x1b[33;1m";
    pub const PINK: &str = "\x1b[38;5;200m";
    pub const GREEN: &str = "\x1b[32;1m";
    pub const RED: &str = "\x1b[31;1m";
}

/// Print colored text to stdout
pub fn print_colored_text(text: &str, color: Option<&str>) {
    match color {
        Some(color) => println!("{}{}{}", color, text, colors::RESET),
        None => println!("{text}"),
    }
}

/// Print bold text to stdout
pub fn print_bold_text(text: &str) {
    println!("{}{}{}", colors::BOLD, text, colors::RESET);
}

/// Get color mapping for items
pub fn get_color_mapping(
    items: &[String],
    excluded_colors: Option<&[String]>,
) -> std::collections::HashMap<String, String> {
    let available_colors = vec![
        "blue".to_string(),
        "yellow".to_string(),
        "pink".to_string(),
        "green".to_string(),
        "red".to_string(),
    ];

    let mut colors = available_colors;
    if let Some(excluded) = excluded_colors {
        colors.retain(|c| !excluded.contains(c));
    }

    let mut mapping = std::collections::HashMap::new();
    for (i, item) in items.iter().enumerate() {
        let color = &colors[i % colors.len()];
        mapping.insert(item.clone(), color.clone());
    }

    mapping
}

/// Get colored text
pub fn get_colored_text(text: &str, color: &str) -> String {
    let color_code = match color {
        "blue" => colors::BLUE,
        "yellow" => colors::YELLOW,
        "pink" => colors::PINK,
        "green" => colors::GREEN,
        "red" => colors::RED,
        _ => colors::RESET,
    };
    format!("{}{}{}", color_code, text, colors::RESET)
}

/// Get bolded text
pub fn get_bolded_text(text: &str) -> String {
    format!("{}{}{}", colors::BOLD, text, colors::RESET)
}
