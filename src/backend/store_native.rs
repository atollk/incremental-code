use crate::backend::backend::StorageBackend;
use serde::{Deserialize, Serialize};

const APP_NAME: &'static str = "incremental-code";

pub struct StoreNative {
    directory: std::path::PathBuf,
}

impl Default for StoreNative {
    fn default() -> Self {
        StoreNative {
            directory: directories::ProjectDirs::from("", "", APP_NAME)
                .unwrap()
                .data_dir()
                .to_path_buf(),
        }
    }
}

impl StorageBackend for StoreNative {
    fn save<T: Serialize>(&self, key: &str, data: &T) -> Result<(), String> {
        let path = self.directory.join(format!("{key}.json"));
        let json = serde_json::to_string_pretty(data).map_err(|e| e.to_string())?;
        std::fs::write(path, json).map_err(|e| e.to_string())
    }

    fn load<T: for<'a> Deserialize<'a>>(&self, key: &str) -> Result<Option<T>, String> {
        let path = self.directory.join(format!("{key}.json"));
        if !path.exists() {
            return Ok(None);
        }
        let json = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&json)
            .map(Some)
            .map_err(|e| e.to_string())
    }

    fn delete(&self, key: &str) -> Result<(), String> {
        let path = self.directory.join(format!("{key}.json"));
        if path.exists() {
            std::fs::remove_file(path).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}
