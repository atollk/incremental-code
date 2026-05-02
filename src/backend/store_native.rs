use crate::backend::backend::StorageBackend;
use serde::{Deserialize, Serialize};
use std::path;

const APP_NAME: &'static str = "incremental-code";

pub struct StoreNative {
    directory: path::PathBuf,
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
    fn save<T: Serialize>(&self, key: &str, data: &T) -> anyhow::Result<()> {
        let path = self.directory.join(format!("{key}.json"));
        let json = serde_json::to_string_pretty(data)?;
        std::fs::create_dir_all(path.parent().unwrap())?;
        std::fs::write(&path, json)?;
        log::info!("saved to {:?}", path::absolute(path)?.into_os_string());
        Ok(())
    }

    fn load<T: for<'a> Deserialize<'a>>(&self, key: &str) -> anyhow::Result<Option<T>> {
        let path = self.directory.join(format!("{key}.json"));
        if !path.exists() {
            return Ok(None);
        }
        let json = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&json).map(Some)?)
    }

    fn delete(&self, key: &str) -> anyhow::Result<()> {
        let path = self.directory.join(format!("{key}.json"));
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }
}
