mod backend;
mod cli;
mod settings;
mod tui;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands, OutputFormat};
use prefer::ConfigValue;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let backend = backend::create_backend(cli.backend);

    if cli.show_paths {
        let paths = backend.search_paths()?;
        print_search_paths(&paths, cli.format);
        return Ok(());
    }

    let file = cli.file.as_ref().ok_or_else(|| {
        anyhow::anyhow!("CONFIG argument required. Use --help for usage.")
    })?;

    if cli.interactive {
        return tui::run(file, backend.as_ref());
    }

    match &cli.command {
        Some(Commands::Get { key }) => {
            let value = backend.get(file, key)?;
            print_value(value.as_ref(), cli.format);
        }
        Some(Commands::Set { key, value }) => {
            backend.set(file, key, value)?;
            if cli.verbose {
                eprintln!("Set {} = {}", key, value);
            }
        }
        Some(Commands::Keys { path }) => {
            let keys = backend.keys(file, path.as_deref())?;
            print_keys(&keys, cli.format);
        }
        Some(Commands::Info) => {
            let info = backend.info(file)?;
            print_info(&info, cli.format);
        }
        Some(Commands::Validate) => {
            let errors = backend.validate(file)?;
            print_validation(&errors, cli.format);
            if !errors.is_empty() {
                std::process::exit(1);
            }
        }
        None => {
            if let Some((key, value)) = cli.parse_key_value() {
                if let Some(val) = value {
                    backend.set(file, key, val)?;
                    if cli.verbose {
                        eprintln!("Set {} = {}", key, val);
                    }
                } else {
                    let result = backend.get(file, key)?;
                    print_value(result.as_ref(), cli.format);
                }
            } else {
                let config = backend.load(file)?;
                print_value(Some(&config), cli.format);
            }
        }
    }

    Ok(())
}

fn print_value(value: Option<&ConfigValue>, format: OutputFormat) {
    match value {
        Some(v) => match format {
            OutputFormat::Json => println!("{}", format_json(v, 0)),
            OutputFormat::Raw => print_raw_value(v),
            OutputFormat::Text => print_text_value(v, 0),
        },
        None => {
            if matches!(format, OutputFormat::Json) {
                println!("null");
            }
        }
    }
}

fn print_raw_value(value: &ConfigValue) {
    match value {
        ConfigValue::String(s) => print!("{}", s),
        ConfigValue::Integer(n) => print!("{}", n),
        ConfigValue::Float(f) => print!("{}", f),
        ConfigValue::Bool(b) => print!("{}", b),
        ConfigValue::Null => {}
        ConfigValue::Array(arr) => {
            for item in arr {
                print_raw_value(item);
                println!();
            }
        }
        ConfigValue::Object(_) => print!("{}", format_json(value, 0)),
    }
}

fn print_text_value(value: &ConfigValue, indent: usize) {
    let prefix = "  ".repeat(indent);
    match value {
        ConfigValue::Object(obj) => {
            let mut keys: Vec<_> = obj.keys().collect();
            keys.sort();
            for key in keys {
                let val = obj.get(key).unwrap();
                match val {
                    ConfigValue::Object(_) | ConfigValue::Array(_) => {
                        println!("{}{}:", prefix, key);
                        print_text_value(val, indent + 1);
                    }
                    _ => {
                        println!("{}{}: {}", prefix, key, format_scalar(val));
                    }
                }
            }
        }
        ConfigValue::Array(arr) => {
            for (i, val) in arr.iter().enumerate() {
                match val {
                    ConfigValue::Object(_) | ConfigValue::Array(_) => {
                        println!("{}[{}]:", prefix, i);
                        print_text_value(val, indent + 1);
                    }
                    _ => {
                        println!("{}[{}]: {}", prefix, i, format_scalar(val));
                    }
                }
            }
        }
        _ => println!("{}{}", prefix, format_scalar(value)),
    }
}

fn format_scalar(value: &ConfigValue) -> String {
    match value {
        ConfigValue::String(s) => s.clone(),
        ConfigValue::Integer(n) => n.to_string(),
        ConfigValue::Float(f) => f.to_string(),
        ConfigValue::Bool(b) => b.to_string(),
        ConfigValue::Null => "null".to_string(),
        _ => format_json(value, 0),
    }
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

fn print_keys(keys: &[String], format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            let items: Vec<String> = keys.iter().map(|k| format!("\"{}\"", k)).collect();
            println!("[\n  {}\n]", items.join(",\n  "));
        }
        OutputFormat::Raw | OutputFormat::Text => {
            for key in keys {
                println!("{}", key);
            }
        }
    }
}

fn print_search_paths(paths: &[String], format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            let items: Vec<String> = paths.iter().map(|p| format!("\"{}\"", p)).collect();
            println!("[\n  {}\n]", items.join(",\n  "));
        }
        OutputFormat::Raw | OutputFormat::Text => {
            for path in paths {
                println!("{}", path);
            }
        }
    }
}

fn print_info(info: &backend::ConfigInfo, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            let search_paths: Vec<String> = info
                .search_paths
                .iter()
                .map(|p| format!("\"{}\"", p))
                .collect();
            println!("{{");
            println!("  \"path\": \"{}\",", info.path);
            println!("  \"format\": \"{}\",", info.format);
            println!("  \"search_paths\": [");
            if !search_paths.is_empty() {
                println!("    {}", search_paths.join(",\n    "));
            }
            println!("  ]");
            println!("}}");
        }
        OutputFormat::Raw | OutputFormat::Text => {
            println!("Path: {}", info.path);
            println!("Format: {}", info.format);
            if !info.search_paths.is_empty() {
                println!("Search paths:");
                for path in &info.search_paths {
                    println!("  {}", path);
                }
            }
        }
    }
}

fn print_validation(errors: &[String], format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            let error_items: Vec<String> = errors.iter().map(|e| format!("\"{}\"", e)).collect();
            println!("{{");
            println!("  \"valid\": {},", errors.is_empty());
            println!("  \"errors\": [");
            if !error_items.is_empty() {
                println!("    {}", error_items.join(",\n    "));
            }
            println!("  ]");
            println!("}}");
        }
        OutputFormat::Raw | OutputFormat::Text => {
            if errors.is_empty() {
                println!("Valid");
            } else {
                println!("Invalid:");
                for error in errors {
                    println!("  - {}", error);
                }
            }
        }
    }
}
