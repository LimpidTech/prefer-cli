use super::{ConfigBackend, ConfigInfo};
use anyhow::{anyhow, Result};
use prefer::ConfigValue;
use std::path::Path;
use std::process::Command;

/// External backend that shells out to prefer implementations
pub struct ExternalBackend {
    /// The command to run (e.g., "prefer", "node", "python3")
    command: String,
    /// Additional arguments before the file path (e.g., ["prefer.js"] for node)
    prefix_args: Vec<String>,
    /// Backend name for error messages
    name: String,
}

impl ExternalBackend {
    pub fn new_rust() -> Self {
        Self {
            command: "prefer".to_string(),
            prefix_args: vec![],
            name: "rust".to_string(),
        }
    }

    pub fn new_js() -> Self {
        Self {
            command: "node".to_string(),
            prefix_args: vec!["prefer.js".to_string()],
            name: "js".to_string(),
        }
    }

    pub fn new_go() -> Self {
        Self {
            command: "prefer".to_string(),
            prefix_args: vec![],
            name: "go".to_string(),
        }
    }

    pub fn new_py() -> Self {
        Self {
            command: "python3".to_string(),
            prefix_args: vec!["-m".to_string(), "prefer".to_string()],
            name: "py".to_string(),
        }
    }

    fn run_command(&self, args: &[&str]) -> Result<String> {
        let mut cmd = Command::new(&self.command);

        for arg in &self.prefix_args {
            cmd.arg(arg);
        }

        for arg in args {
            cmd.arg(arg);
        }

        let output = cmd
            .output()
            .map_err(|e| anyhow!("{} backend failed to execute: {}", self.name, e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!(
                "{} backend returned error: {}",
                self.name,
                stderr.trim()
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn parse_json(&self, output: &str) -> Result<ConfigValue> {
        prefer::formats::parse(output, Path::new("response.json"))
            .map_err(|e| anyhow!("Failed to parse JSON output: {}", e))
    }

    fn parse_string_array(&self, output: &str) -> Result<Vec<String>> {
        let value = self.parse_json(output)?;
        match value.as_array() {
            Some(arr) => arr
                .iter()
                .map(|v| {
                    v.as_str()
                        .map(|s| s.to_string())
                        .ok_or_else(|| anyhow!("Expected string in array"))
                })
                .collect(),
            None => Err(anyhow!("Expected array")),
        }
    }
}

impl ConfigBackend for ExternalBackend {
    fn load(&self, path: &Path) -> Result<ConfigValue> {
        let path_str = path.to_string_lossy();
        let output = self.run_command(&["load", &path_str])?;
        self.parse_json(&output)
    }

    fn get(&self, path: &Path, key: &str) -> Result<Option<ConfigValue>> {
        let path_str = path.to_string_lossy();
        let output = self.run_command(&["get", &path_str, key])?;

        if output.trim().is_empty() {
            return Ok(None);
        }

        Ok(Some(self.parse_json(&output)?))
    }

    fn set(&self, path: &Path, key: &str, value: &str) -> Result<()> {
        let path_str = path.to_string_lossy();
        self.run_command(&["set", &path_str, key, value])?;
        Ok(())
    }

    fn keys(&self, path: &Path, prefix: Option<&str>) -> Result<Vec<String>> {
        let path_str = path.to_string_lossy();
        let output = match prefix {
            Some(p) => self.run_command(&["keys", &path_str, p])?,
            None => self.run_command(&["keys", &path_str])?,
        };

        if output.trim().is_empty() {
            return Ok(vec![]);
        }

        self.parse_string_array(&output)
    }

    fn info(&self, path: &Path) -> Result<ConfigInfo> {
        let path_str = path.to_string_lossy();
        let output = self.run_command(&["info", &path_str])?;

        let value = self.parse_json(&output)?;
        let obj = value
            .as_object()
            .ok_or_else(|| anyhow!("Expected object for info"))?;

        let path = obj
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing path in info"))?
            .to_string();

        let format = obj
            .get("format")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing format in info"))?
            .to_string();

        let search_paths = obj
            .get("search_paths")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(ConfigInfo {
            path,
            format,
            search_paths,
        })
    }

    fn validate(&self, path: &Path) -> Result<Vec<String>> {
        let path_str = path.to_string_lossy();
        let output = self.run_command(&["validate", &path_str])?;

        if output.trim().is_empty() {
            return Ok(vec![]);
        }

        self.parse_string_array(&output)
    }

    fn search_paths(&self) -> Result<Vec<String>> {
        let output = self.run_command(&["search-paths"])?;

        if output.trim().is_empty() {
            return Ok(vec![]);
        }

        self.parse_string_array(&output)
    }
}
