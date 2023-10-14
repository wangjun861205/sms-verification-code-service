use crate::core::locker::Locker;
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Default)]
pub struct MemoryLocker {
    states: Mutex<HashMap<String, Arc<Mutex<bool>>>>,
}

impl MemoryLocker {
    pub fn new() -> Self {
        MemoryLocker {
            states: Mutex::new(HashMap::new()),
        }
    }

    pub async fn get_or_insert_state(&self, key: &str) -> Arc<Mutex<bool>> {
        self.states
            .lock()
            .await
            .entry(key.to_owned())
            .or_insert(Arc::new(Mutex::new(false)))
            .clone()
    }
}

impl Locker for MemoryLocker {
    async fn lock(&mut self, phone: &str) -> Result<(), Box<dyn Display>> {
        let state = self.get_or_insert_state(phone).await;
        let mut state = state.lock().await;
        *state = true;
        Ok(())
    }

    async fn unlock(&mut self, phone: &str) -> Result<(), Box<dyn Display>> {
        let states = self.states.lock().await;
        let state = states
            .get(phone)
            .ok_or(Box::new("phone not found".to_owned()) as Box<dyn Display>)?;
        *state.lock().await = false;
        Ok(())
    }
}
