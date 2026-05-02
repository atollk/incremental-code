use crate::backend::backend::StorageBackend;
use serde::{Deserialize, Serialize};

pub struct StoreWeb {}

impl Default for StoreWeb {
    fn default() -> Self {
        Self {}
    }
}

impl StorageBackend for StoreWeb {
    fn save<T: Serialize>(&self, key: &str, data: &T) -> Result<(), String> {
        let storage = get_local_storage()?;
        let json = serde_json::to_string(data).map_err(|e| e.to_string())?;
        storage
            .set_item(&format!("save_{key}"), &json)
            .map_err(|e| format!("{e:?}"))
    }

    fn load<T: for<'a> Deserialize<'a>>(&self, key: &str) -> Result<Option<T>, String> {
        let storage = get_local_storage()?;
        match storage
            .get_item(&format!("save_{key}"))
            .map_err(|e| format!("{e:?}"))?
        {
            None => Ok(None),
            Some(json) => serde_json::from_str(&json)
                .map(Some)
                .map_err(|e| e.to_string()),
        }
    }

    fn delete(&self, key: &str) -> Result<(), String> {
        let storage = get_local_storage()?;
        storage
            .remove_item(&format!("save_{key}"))
            .map_err(|e| format!("{e:?}"))
    }
}

fn get_local_storage() -> Result<web_sys::Storage, String> {
    web_sys::window()
        .ok_or("No window")?
        .local_storage()
        .map_err(|e| format!("{e:?}"))?
        .ok_or_else(|| "localStorage unavailable".to_string())
}
