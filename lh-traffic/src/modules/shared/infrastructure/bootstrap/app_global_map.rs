use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::any::Any;
use once_cell::sync::Lazy;
use super::app_key_enum::AppKeyEnum;

/// Global application context for storing singleton instances
/// Similar to the TypeScript AppGlobalMap
pub struct AppGlobalMap {
    dictionary: Arc<RwLock<HashMap<String, Box<dyn Any + Send + Sync>>>>,
}

static INSTANCE: Lazy<Arc<RwLock<AppGlobalMap>>> = Lazy::new(|| {
    Arc::new(RwLock::new(AppGlobalMap {
        dictionary: Arc::new(RwLock::new(HashMap::new())),
    }))
});

impl AppGlobalMap {
    pub fn instance() -> Arc<RwLock<AppGlobalMap>> {
        INSTANCE.clone()
    }

    pub fn set<T: Any + Send + Sync + 'static>(&self, key: AppKeyEnum, value: T) {
        let mut dict = self.dictionary.write().unwrap();
        dict.insert(key.to_string(), Box::new(value));
    }

    pub fn get<T: Any + Send + Sync + Clone + 'static>(&self, key: AppKeyEnum) -> Option<T> {
        let dict = self.dictionary.read().unwrap();
        dict.get(&key.to_string())
            .and_then(|boxed| boxed.downcast_ref::<T>())
            .cloned()
    }

    pub fn has(&self, key: AppKeyEnum) -> bool {
        let dict = self.dictionary.read().unwrap();
        dict.contains_key(&key.to_string())
    }
}
