use crate::backend::backend::StorageBackend;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};

pub struct StoreWeb {}

impl Default for StoreWeb {
    fn default() -> Self {
        Self {}
    }
}

impl StorageBackend for StoreWeb {
    fn save<T: Serialize>(&self, key: &str, data: &T) -> anyhow::Result<()> {
        let storage = get_local_storage()?;
        let json = serde_json::to_string(data).map_err(|e| anyhow!(format!("{e:?}")))?;
        storage
            .set_item(&format!("save_{key}"), &json)
            .map_err(|e| anyhow!(format!("{e:?}")))?;
        log::info!("saved to local storage with key: {key}");
        Ok(())
    }

    fn load<T: for<'a> Deserialize<'a>>(&self, key: &str) -> anyhow::Result<Option<T>> {
        let storage = get_local_storage()?;
        let result = match storage
            .get_item(&format!("save_{key}"))
            .map_err(|e| anyhow!(format!("{e:?}")))?
        {
            None => None,
            Some(json) => serde_json::from_str(&json).map(Some)?,
        };
        log::info!("loaded from local storage with key: {key}");
        Ok(result)
    }

    fn delete(&self, key: &str) -> anyhow::Result<()> {
        let storage = get_local_storage()?;
        storage
            .remove_item(&format!("save_{key}"))
            .map_err(|e| anyhow!(format!("{e:?}")))
    }
}

fn get_local_storage() -> anyhow::Result<web_sys::Storage> {
    web_sys::window()
        .ok_or(anyhow!("No window"))?
        .local_storage()
        .map_err(|e| anyhow!(format!("{e:?}")))?
        .ok_or_else(|| anyhow!("localStorage unavailable"))
}
