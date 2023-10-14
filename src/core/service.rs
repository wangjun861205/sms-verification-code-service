use std::fmt::Display;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::core::{generator::Generator, sender::Sender, store::Store};
use std::collections::HashMap;

#[derive(Debug)]
pub struct _Service<SD, ST, GN>
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
    lockers: HashMap<String, Arc<Mutex<()>>>,
}

#[derive(Debug, Clone)]
pub struct Service<SD, ST, GN>(Arc<Mutex<_Service<SD, ST, GN>>>)
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
        Self(Arc::new(Mutex::new(_Service {
            min_interval,
            expire_after,
            sender,
            store,
            generator,
            lockers: HashMap::new(),
        })))
    }

    pub async fn send_service(&self, phone: &str) -> SendService<SD, ST, GN> {
        let mut service = self.0.lock().await;
        if let Some(locker) = service.lockers.get(phone) {
            let locker = locker.clone();
            return SendService {
                phone: phone.to_owned(),
                min_interval: service.min_interval,
                sender: service.sender.clone(),
                store: service.store.clone(),
                generator: service.generator.clone(),
                locker,
            };
        }
        let locker = Arc::new(Mutex::new(()));
        service.lockers.insert(phone.to_owned(), locker.clone());
        SendService {
            phone: phone.to_owned(),
            min_interval: service.min_interval,
            sender: service.sender.clone(),
            store: service.store.clone(),
            generator: service.generator.clone(),
            locker,
        }
    }

    pub async fn verify_service(&self, phone: &str, code: &str) -> VerifyService<ST> {
        let mut service = self.0.lock().await;
        if let Some(locker) = service.lockers.get(phone) {
            let locker = locker.clone();
            return VerifyService {
                phone: phone.to_owned(),
                code: code.to_owned(),
                expire_after: service.expire_after,
                store: service.store.clone(),
                locker,
            };
        }
        let locker = Arc::new(Mutex::new(()));
        service.lockers.insert(phone.to_owned(), locker.clone());
        VerifyService {
            phone: phone.to_owned(),
            code: "".to_owned(),
            expire_after: service.expire_after,
            store: service.store.clone(),
            locker,
        }
    }
}

pub struct SendService<SD, ST, GN>
where
    SD: Sender,
    ST: Store,
    GN: Generator,
{
    phone: String,
    min_interval: i64,
    sender: SD,
    store: ST,
    generator: GN,
    locker: Arc<Mutex<()>>,
}

impl<SD, ST, GN> SendService<SD, ST, GN>
where
    SD: Sender,
    ST: Store,
    GN: Generator,
{
    pub async fn send_code(&mut self) -> Result<(), Box<dyn Display>> {
        let _ = self.locker.lock().await;
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
}

pub struct VerifyService<ST>
where
    ST: Store,
{
    phone: String,
    code: String,
    expire_after: i64,
    store: ST,
    locker: Arc<Mutex<()>>,
}

impl<ST> VerifyService<ST>
where
    ST: Store,
{
    pub async fn check_code(&mut self) -> Result<bool, Box<dyn Display>> {
        let _ = self.locker.lock().await;
        let is_ok = self
            .store
            .get(&self.phone)
            .await?
            .map_or(Ok(false), |stored_code| {
                Ok(
                    chrono::Utc::now().timestamp() <= stored_code.sent_at + self.expire_after
                        && stored_code.code == self.code,
                )
            })?;
        Ok(is_ok)
    }
}
