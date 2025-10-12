//! UCI options configuration system for Scacchista

/// UCI option types
#[derive(Debug, Clone, PartialEq, Eq)]
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

    /// Whether to use experience book
    pub use_experience_book: bool,

    /// Experience book file path
    pub experience_book_path: Option<String>,

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
            experience_book_path: None,
            chess_style: "Normal".to_string(),
            analyze_mode: false,
            debug_log: false,
            engine_name: "Scacchista".to_string(),
            author: "Claude Code".to_string(),
        }
    }
}

impl UciOptions {
    /// Create new options with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set option value (basic implementation)
    pub fn set_option(&mut self, name: &str, value: Option<&str>) -> Result<(), String> {
        match name {
            "Hash" => {
                if let Some(v_str) = value {
                    if let Ok(val) = v_str.parse::<u64>() {
                        self.hash = val;
                    } else {
                        return Err(format!("Invalid numeric value for Hash: {}", v_str));
                    }
                }
            }
            "Threads" => {
                if let Some(v_str) = value {
                    if let Ok(val) = v_str.parse::<u8>() {
                        self.threads = val;
                    } else {
                        return Err(format!("Invalid numeric value for Threads: {}", v_str));
                    }
                }
            }
            "SyzygyPath" => {
                self.syzygy_path = value.map(|s| s.to_string());
            }
            "UseExperienceBook" => {
                if let Some(v_str) = value {
                    let v = matches!(v_str.to_lowercase().as_str(), "true" | "1" | "yes");
                    self.use_experience_book = v;
                }
            }
            "Style" => {
                if let Some(v_str) = value {
                    self.chess_style = v_str.to_string();
                }
            }
            _ => {
                return Err(format!("Unknown option: {}", name));
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
