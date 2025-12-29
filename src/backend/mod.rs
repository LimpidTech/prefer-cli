mod native;
mod external;

pub use native::NativeBackend;
pub use external::ExternalBackend;

use crate::cli::Backend;
use anyhow::Result;
use prefer::ConfigValue;
use std::path::Path;

/// Information about a configuration file
#[derive(Debug, Clone)]
pub struct ConfigInfo {
    /// The resolved file path
    pub path: String,
    /// The detected format (json, yaml, toml, etc.)
    pub format: String,
    /// Search paths that were checked
    pub search_paths: Vec<String>,
}

/// Trait for configuration backends
pub trait ConfigBackend: Send + Sync {
    /// Load and parse a configuration file
    fn load(&self, path: &Path) -> Result<ConfigValue>;

    /// Get a value at a specific key path
    fn get(&self, path: &Path, key: &str) -> Result<Option<ConfigValue>>;

    /// Set a value at a specific key path
    fn set(&self, path: &Path, key: &str, value: &str) -> Result<()>;

    /// List keys at a given path
    fn keys(&self, path: &Path, prefix: Option<&str>) -> Result<Vec<String>>;

    /// Get configuration file info
    fn info(&self, path: &Path) -> Result<ConfigInfo>;

    /// Validate the configuration file
    fn validate(&self, path: &Path) -> Result<Vec<String>>;

    /// Get the search paths prefer would check
    fn search_paths(&self) -> Result<Vec<String>>;
}

/// Create a backend based on the CLI selection
pub fn create_backend(backend: Backend) -> Box<dyn ConfigBackend> {
    match backend {
        Backend::Native => Box::new(NativeBackend::new()),
        Backend::Rust => Box::new(ExternalBackend::new_rust()),
        Backend::Js => Box::new(ExternalBackend::new_js()),
        Backend::Go => Box::new(ExternalBackend::new_go()),
        Backend::Py => Box::new(ExternalBackend::new_py()),
    }
}
