use super::{ConfigBackend, ConfigInfo};
use anyhow::{anyhow, Result};
use prefer::discovery::{find_config_file, get_search_paths};
use prefer::{ConfigBuilder, ConfigValue};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::runtime::Runtime;

pub struct NativeBackend {
    runtime: Runtime,
}

impl NativeBackend {
    pub fn new() -> Self {
        Self {
            runtime: Runtime::new().expect("Failed to create tokio runtime"),
        }
    }

    fn resolve_path(&self, path: &Path) -> Result<PathBuf> {
        if path.exists() {
            return Ok(path.to_path_buf());
        }

        let name = path.to_string_lossy();
        self.runtime
            .block_on(find_config_file(&name))
            .map_err(|e| anyhow!("Config not found: {}", e))
    }
}

impl Default for NativeBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigBackend for NativeBackend {
    fn load(&self, path: &Path) -> Result<ConfigValue> {
        let resolved = self.resolve_path(path)?;

        let config = self.runtime.block_on(async {
            ConfigBuilder::new()
                .add_file(&resolved)
                .build()
                .await
                .map_err(|e| anyhow!("Failed to load config: {}", e))
        })?;

        Ok(config.data().clone())
    }

    fn get(&self, path: &Path, key: &str) -> Result<Option<ConfigValue>> {
        let config = self.load(path)?;

        let parts: Vec<&str> = key.split('.').collect();
        let mut current = &config;

        for part in parts {
            match current.get(part) {
                Some(v) => current = v,
                None => return Ok(None),
            }
        }

        Ok(Some(current.clone()))
    }

    fn set(&self, path: &Path, key: &str, value: &str) -> Result<()> {
        let resolved = self.resolve_path(path)?;
        let mut config = self.load(path)?;

        let parsed_value = parse_value(value);

        let parts: Vec<&str> = key.split('.').collect();

        if parts.is_empty() {
            return Err(anyhow!("Empty key path"));
        }

        set_nested(&mut config, &parts, parsed_value)?;

        let format = detect_format(&resolved)?;
        write_config(&resolved, &config, &format)?;

        Ok(())
    }

    fn keys(&self, path: &Path, prefix: Option<&str>) -> Result<Vec<String>> {
        let config = self.load(path)?;

        let target = if let Some(prefix) = prefix {
            let parts: Vec<&str> = prefix.split('.').collect();
            let mut current = &config;
            for part in parts {
                match current.get(part) {
                    Some(v) => current = v,
                    None => return Ok(vec![]),
                }
            }
            current
        } else {
            &config
        };

        match target.as_object() {
            Some(obj) => Ok(obj.keys().cloned().collect()),
            None => Ok(vec![]),
        }
    }

    fn info(&self, path: &Path) -> Result<ConfigInfo> {
        let resolved = self.resolve_path(path)?;
        let format = detect_format(&resolved)?;
        let canonical = resolved.canonicalize().unwrap_or(resolved);

        Ok(ConfigInfo {
            path: canonical.to_string_lossy().to_string(),
            format,
            search_paths: self.search_paths()?,
        })
    }

    fn validate(&self, path: &Path) -> Result<Vec<String>> {
        let mut errors = vec![];

        match self.load(path) {
            Ok(_) => {}
            Err(e) => errors.push(format!("Parse error: {}", e)),
        }

        Ok(errors)
    }

    fn search_paths(&self) -> Result<Vec<String>> {
        Ok(get_search_paths()
            .into_iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect())
    }
}

fn parse_value(s: &str) -> ConfigValue {
    if s == "null" {
        return ConfigValue::Null;
    }
    if s == "true" {
        return ConfigValue::Bool(true);
    }
    if s == "false" {
        return ConfigValue::Bool(false);
    }
    if let Ok(n) = s.parse::<i64>() {
        return ConfigValue::Integer(n);
    }
    if let Ok(f) = s.parse::<f64>() {
        return ConfigValue::Float(f);
    }
    ConfigValue::String(s.to_string())
}

fn set_nested(config: &mut ConfigValue, parts: &[&str], value: ConfigValue) -> Result<()> {
    if parts.is_empty() {
        return Err(anyhow!("Empty key path"));
    }

    let mut current = config;
    for part in &parts[..parts.len() - 1] {
        current = current
            .as_object_mut()
            .ok_or_else(|| anyhow!("Path component '{}' is not an object", part))?
            .entry(part.to_string())
            .or_insert(ConfigValue::Object(HashMap::new()));
    }

    let last_key = parts.last().unwrap();
    current
        .as_object_mut()
        .ok_or_else(|| anyhow!("Cannot set value on non-object"))?
        .insert(last_key.to_string(), value);

    Ok(())
}

fn detect_format(path: &Path) -> Result<String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let format = match ext.to_lowercase().as_str() {
        "json" | "json5" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "ini" | "cfg" => "ini",
        "xml" => "xml",
        _ => return Err(anyhow!("Unknown format for extension: {}", ext)),
    };

    Ok(format.to_string())
}

fn write_config(path: &Path, config: &ConfigValue, format: &str) -> Result<()> {
    let content = match format {
        "json" => format_json(config, 0),
        "yaml" => return Err(anyhow!("YAML write not yet implemented")),
        "toml" => return Err(anyhow!("TOML write not yet implemented")),
        _ => return Err(anyhow!("Write not supported for format: {}", format)),
    };

    std::fs::write(path, content)?;
    Ok(())
}

fn format_json(value: &ConfigValue, indent: usize) -> String {
    let spaces = "  ".repeat(indent);
    let inner_spaces = "  ".repeat(indent + 1);

    match value {
        ConfigValue::Null => "null".to_string(),
        ConfigValue::Bool(b) => b.to_string(),
        ConfigValue::Integer(n) => n.to_string(),
        ConfigValue::Float(f) => f.to_string(),
        ConfigValue::String(s) => format!("\"{}\"", escape_json_string(s)),
        ConfigValue::Array(arr) => {
            if arr.is_empty() {
                "[]".to_string()
            } else {
                let items: Vec<String> = arr
                    .iter()
                    .map(|v| format!("{}{}", inner_spaces, format_json(v, indent + 1)))
                    .collect();
                format!("[\n{}\n{}]", items.join(",\n"), spaces)
            }
        }
        ConfigValue::Object(obj) => {
            if obj.is_empty() {
                "{}".to_string()
            } else {
                let mut keys: Vec<_> = obj.keys().collect();
                keys.sort();
                let items: Vec<String> = keys
                    .iter()
                    .map(|k| {
                        format!(
                            "{}\"{}\": {}",
                            inner_spaces,
                            escape_json_string(k),
                            format_json(obj.get(*k).unwrap(), indent + 1)
                        )
                    })
                    .collect();
                format!("{{\n{}\n{}}}", items.join(",\n"), spaces)
            }
        }
    }
}

fn escape_json_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c.is_control() => result.push_str(&format!("\\u{:04x}", c as u32)),
            c => result.push(c),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_json() {
        let mut file = NamedTempFile::with_suffix(".json").unwrap();
        writeln!(file, r#"{{"name": "test", "port": 8080}}"#).unwrap();

        let backend = NativeBackend::new();
        let config = backend.load(file.path()).unwrap();

        assert_eq!(config.get("name").unwrap().as_str(), Some("test"));
        assert_eq!(config.get("port").unwrap().as_i64(), Some(8080));
    }

    #[test]
    fn test_get_nested() {
        let mut file = NamedTempFile::with_suffix(".json").unwrap();
        writeln!(file, r#"{{"database": {{"host": "localhost", "port": 5432}}}}"#).unwrap();

        let backend = NativeBackend::new();
        let value = backend.get(file.path(), "database.host").unwrap();

        assert_eq!(value.unwrap().as_str(), Some("localhost"));
    }

    #[test]
    fn test_keys() {
        let mut file = NamedTempFile::with_suffix(".json").unwrap();
        writeln!(file, r#"{{"a": 1, "b": 2, "c": 3}}"#).unwrap();

        let backend = NativeBackend::new();
        let keys = backend.keys(file.path(), None).unwrap();

        assert!(keys.contains(&"a".to_string()));
        assert!(keys.contains(&"b".to_string()));
        assert!(keys.contains(&"c".to_string()));
    }
}
