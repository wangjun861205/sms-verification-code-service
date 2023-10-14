use crate::core::{generator::Generator, sender::Sender, store::Store};
use std::fmt::Display;

use super::locker::Locker;

pub struct Service<SD, ST, GN, LK>
where
    SD: Sender,
    ST: Store,
    GN: Generator,
    LK: Locker,
{
    min_interval: i64,
    expire_after: i64,
    sender: SD,
    store: ST,
    generator: GN,
    locker: LK,
}

impl<SD, ST, GN, LK> Service<SD, ST, GN, LK>
where
    SD: Sender,
    ST: Store,
    GN: Generator,
    LK: Locker,
{
    pub fn new(
        min_interval: i64,
        expire_after: i64,
        sender: SD,
        store: ST,
        generator: GN,
        locker: LK,
    ) -> Self {
        Self {
            min_interval,
            expire_after,
            sender,
            store,
            generator,
            locker,
        }
    }

    pub async fn send_code(&mut self, phone: &str) -> Result<(), Box<dyn Display>> {
        self.locker.lock(phone).await?;
        if let Some(latest) = self.store.get(phone).await? {
            if chrono::Utc::now().timestamp() - latest.sent_at < self.min_interval {
                return Err(Box::new("Too frequent".to_owned()) as Box<dyn Display>);
            }
        }
        let code = self.generator.generate();
        self.sender.send(phone, &code).await?;
        self.store.put(phone, &code).await?;
        self.locker.unlock(phone).await
    }

    pub async fn check_code(&mut self, phone: &str, code: &str) -> Result<bool, Box<dyn Display>> {
        self.locker.lock(phone);
        let is_ok = self
            .store
            .get(phone)
            .await?
            .map_or(Ok(false), |stored_code| {
                Ok(
                    chrono::Utc::now().timestamp() <= stored_code.sent_at + self.expire_after
                        && stored_code.code == code,
                )
            })?;
        self.locker.unlock(phone).await?;
        Ok(is_ok)
    }
}
