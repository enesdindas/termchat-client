use std::path::PathBuf;

pub struct Config {
    pub server_url: String,
    pub token_path: PathBuf,
}

impl Config {
    pub fn load() -> Self {
        let server_url = std::env::var("TERMCHAT_SERVER")
            .unwrap_or_else(|_| "https://termchat-server-09qq.onrender.com".to_string());

        let token_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("termchat")
            .join("token");

        Config { server_url, token_path }
    }

    pub fn ws_url(&self) -> String {
        self.server_url
            .replace("https://", "wss://")
            .replace("http://", "ws://")
            + "/ws"
    }

    pub fn save_token(&self, token: &str) -> anyhow::Result<()> {
        if let Some(parent) = self.token_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.token_path, token)?;
        Ok(())
    }

    pub fn load_token(&self) -> Option<String> {
        std::fs::read_to_string(&self.token_path)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    pub fn delete_token(&self) -> anyhow::Result<()> {
        match std::fs::remove_file(&self.token_path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}
