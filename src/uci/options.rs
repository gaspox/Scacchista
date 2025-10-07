//! UCI options configuration system for Scacchista
//!
//! This module provides the complete UCI options system including:
//! - Option definitions with types, defaults, and constraints
//! - Option value storage and type validation
//! - Integration with search parameters and engine behavior

use serde::{Deserialize, Serialize};

/// UCI option types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OptionType {
    Check { default: bool },
    Spin { default: i64, min: i64, max: i64 },
    String { default: String },
}

/// Individual UCI option definition
#[derive(Debug, Clone)]
pub struct UciOption {
    pub name: String,
    pub opt_type: OptionType,
}

/// Complete UCI options configuration
#[derive(Debug, Clone)]
pub struct UciOptions {
    /// Hash table size in MB
    pub hash: u64,

    /// Number of search threads
    pub threads: u8,

    /// Syzygy tablebase path
    pub syzygy_path: Option<String>,

    /// Search style (Normal, Tal, Petrosian)
    pub style: String,

    /// Whether to use experience book
    pub use_experience_book: bool,

    /// Chess style: Normal, Tal, Petrosian
    pub chess_style: String,

    /// Whether to enable UCI_AnalysisMode by default
    pub analyze_mode: bool,

    /// Debug logging enabled
    pub debug_log: bool,

    /// Engine name
    pub engine_name: String,

    /// Engine author
    pub author: String,
}

impl Default for UciOptions {
    fn default() -> Self {
        Self {
            hash: 16,
            threads: 1,
            syzygy_path: None,
            use_experience_book: true,

    /// Experience book file path
    pub experience_book_path: Option<String>,
}

impl UciOptions {
    /// Create new options with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set option value
    pub fn set_option(&mut self, name: &str, value: Option<&str>) -> Result<(), String> {
        match name {
            "Hash" => {
                if let Some(v_str) = value {
                    if let Ok(val) = v_str.parse::<u64>() {
            self.hash = val;
        } else {
            return Err(format!("Invalid value for option {}: {}", name, v_str) {
                }
            }
            "Threads" => {
                if let Some(v_str) = value {
                    if let Ok(val) = v_str.parse::<u64>() {
                    return Err(format!("Option {} requires a numeric value", name));
            }

            // Handle other options
            _ => return Err(format!("Unknown option: {}", name));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_option_defaults() {
        let options = UciOptions::new();
        assert_eq!(options.hash, 16);
        assert_eq!(options.threads, 1);
        assert_eq!(options.engine_name, "Scacchista");
        assert_eq!(options.use_experience_book, true);
        assert_eq!(options.chess_style, "Normal");
        assert_eq!(options.analyze_mode, false);
    }
}