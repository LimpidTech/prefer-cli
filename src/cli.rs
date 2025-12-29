use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Backend {
    /// Use the native Rust prefer library compiled into this binary (default)
    Native,
    /// Shell out to an external prefer.rs binary (for version testing)
    Rust,
    /// Shell out to Node.js prefer.js
    Js,
    /// Shell out to prefer.go binary
    Go,
    /// Shell out to Python prefer.py
    Py,
}

impl Default for Backend {
    fn default() -> Self {
        Self::Native
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable output
    Text,
    /// JSON output
    Json,
    /// Raw value only (no key, no formatting)
    Raw,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Text
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "prefer",
    author = "LimpidTech",
    version,
    about = "A CLI tool for querying, setting, and exploring configuration files",
    long_about = "prefer-cli provides a command-line interface for working with configuration files.\n\n\
                  It supports multiple formats (JSON, YAML, TOML, INI, XML, JSON5) and can use\n\
                  different backends for parsing, making it useful for integration testing.\n\n\
                  The CONFIG argument can be a name (e.g., 'myapp') which prefer will search for\n\
                  in standard paths, or an explicit path to a file."
)]
pub struct Cli {
    /// Configuration name or path (e.g., 'myapp' or './config.json')
    #[arg(value_name = "CONFIG")]
    pub file: Option<PathBuf>,

    /// Key path to query or set (dot-notation, e.g., 'database.host')
    /// If a value is provided after '=', sets the key to that value
    #[arg(value_name = "KEY[=VALUE]")]
    pub key_value: Option<String>,

    /// Launch interactive TUI mode
    #[arg(short, long)]
    pub interactive: bool,

    /// Backend to use for parsing configuration
    #[arg(short, long, value_enum, default_value_t = Backend::Native)]
    pub backend: Backend,

    /// Output format
    #[arg(short = 'f', long, value_enum, default_value_t = OutputFormat::Text)]
    pub format: OutputFormat,

    /// Show search paths that prefer checks for configuration files
    #[arg(long)]
    pub show_paths: bool,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Get a configuration value
    Get {
        /// Key path in dot-notation (e.g., 'database.host')
        key: String,
    },

    /// Set a configuration value
    Set {
        /// Key path in dot-notation (e.g., 'database.host')
        key: String,
        /// Value to set
        value: String,
    },

    /// List all keys at a given path
    Keys {
        /// Path to list keys from (optional, defaults to root)
        path: Option<String>,
    },

    /// Show configuration file metadata
    Info,

    /// Validate configuration file
    Validate,
}

impl Cli {
    /// Parse the key_value argument into separate key and optional value
    pub fn parse_key_value(&self) -> Option<(&str, Option<&str>)> {
        self.key_value.as_ref().map(|kv| {
            if let Some(idx) = kv.find('=') {
                let (key, value) = kv.split_at(idx);
                (key, Some(&value[1..]))
            } else {
                (kv.as_str(), None)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_key_only() {
        let cli = Cli {
            file: Some(PathBuf::from("config.json")),
            key_value: Some("database.host".to_string()),
            interactive: false,
            backend: Backend::Native,
            format: OutputFormat::Text,
            show_paths: false,
            verbose: false,
            command: None,
        };

        let result = cli.parse_key_value();
        assert_eq!(result, Some(("database.host", None)));
    }

    #[test]
    fn test_parse_key_value() {
        let cli = Cli {
            file: Some(PathBuf::from("config.json")),
            key_value: Some("database.host=localhost".to_string()),
            interactive: false,
            backend: Backend::Native,
            format: OutputFormat::Text,
            show_paths: false,
            verbose: false,
            command: None,
        };

        let result = cli.parse_key_value();
        assert_eq!(result, Some(("database.host", Some("localhost"))));
    }

    #[test]
    fn test_parse_no_key_value() {
        let cli = Cli {
            file: Some(PathBuf::from("config.json")),
            key_value: None,
            interactive: false,
            backend: Backend::Native,
            format: OutputFormat::Text,
            show_paths: false,
            verbose: false,
            command: None,
        };

        let result = cli.parse_key_value();
        assert_eq!(result, None);
    }
}
