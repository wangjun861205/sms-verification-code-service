use std::fmt::Display;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::core::{generator::Generator, sender::Sender, store::Store};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Executor<SD, ST, GN>
where
    SD: Sender + Clone,
    ST: Store + Clone,
    GN: Generator + Clone,
{
    phone: String,
    min_interval: i64,
    expire_after: i64,
    sender: SD,
    store: ST,
    generator: GN,
}

impl<SD, ST, GN> Executor<SD, ST, GN>
where
    SD: Sender + Clone,
    ST: Store + Clone,
    GN: Generator + Clone,
{
    pub async fn send_code(&mut self) -> Result<(), Box<dyn Display>> {
        if let Some(latest) = self.store.get(&self.phone).await? {
            if chrono::Utc::now().timestamp() - latest.sent_at < self.min_interval {
                return Err(Box::new("Too frequent".to_owned()) as Box<dyn Display>);
            }
        }
        let code = self.generator.generate();
        self.sender.send(&self.phone, &code).await?;
        self.store.put(&self.phone, &code).await?;
        Ok(())
    }

    pub async fn verify_code(&mut self, code: &str) -> Result<(), Box<dyn Display>> {
        match self.store.get(&self.phone).await? {
            Some(stored_code) => {
                if chrono::Utc::now().timestamp() <= stored_code.sent_at + self.expire_after
                    && stored_code.code == code
                {
                    Ok(())
                } else {
                    Err(Box::new("Invalid code".to_owned()) as Box<dyn Display>)
                }
            }
            None => Err(Box::new("Invalid phone".to_owned()) as Box<dyn Display>),
        }
    }
}

type ExecutorMap<SD, ST, GN> = HashMap<String, Arc<Mutex<Executor<SD, ST, GN>>>>;

#[derive(Debug)]
struct InnerService<SD, ST, GN>
where
    SD: Sender + Clone,
    ST: Store + Clone,
    GN: Generator + Clone,
{
    min_interval: i64,
    expire_after: i64,
    sender: SD,
    store: ST,
    generator: GN,
    executors: ExecutorMap<SD, ST, GN>,
}

#[derive(Debug, Clone)]
pub struct Service<SD, ST, GN>(Arc<Mutex<InnerService<SD, ST, GN>>>)
where
    SD: Sender + Clone,
    ST: Store + Clone,
    GN: Generator + Clone;

impl<SD, ST, GN> Service<SD, ST, GN>
where
    SD: Sender + Clone,
    ST: Store + Clone,
    GN: Generator + Clone,
{
    pub fn new(min_interval: i64, expire_after: i64, sender: SD, store: ST, generator: GN) -> Self {
        Self(Arc::new(Mutex::new(InnerService {
            min_interval,
            expire_after,
            sender,
            store,
            generator,
            executors: HashMap::new(),
        })))
    }

    pub async fn acquire_executor(&self, phone: &str) -> Arc<Mutex<Executor<SD, ST, GN>>> {
        let mut service = self.0.lock().await;
        if let Some(executor) = service.executors.get(phone) {
            return executor.clone();
        }
        let executor = Arc::new(Mutex::new(Executor {
            phone: phone.to_owned(),
            min_interval: service.min_interval,
            expire_after: service.expire_after,
            sender: service.sender.clone(),
            store: service.store.clone(),
            generator: service.generator.clone(),
        }));
        service.executors.insert(phone.to_owned(), executor.clone());
        executor
    }
}
