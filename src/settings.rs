use tokio::runtime::Runtime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    #[default]
    Vi,
    Basic,
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub mode: InputMode,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: InputMode::Vi,
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        let runtime = match Runtime::new() {
            Ok(rt) => rt,
            Err(_) => return Self::default(),
        };

        runtime.block_on(async {
            let config = match prefer::load("prefer").await {
                Ok(c) => c,
                Err(_) => return Self::default(),
            };

            let mode = config
                .data()
                .get("mode")
                .and_then(|v| v.as_str())
                .map(|s| match s.to_lowercase().as_str() {
                    "basic" => InputMode::Basic,
                    "vi" | "vim" => InputMode::Vi,
                    _ => InputMode::Vi,
                })
                .unwrap_or(InputMode::Vi);

            Self { mode }
        })
    }
}
