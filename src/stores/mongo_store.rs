use std::fmt::Display;

use chrono::Utc;
use mongodb::{bson::doc, options::UpdateOptions, Collection};

use crate::core::{entites::SMSVerificationCode, store::Store};

#[derive(Debug)]
pub struct MongoStore {
    collection: Collection<SMSVerificationCode>,
}

impl MongoStore {
    pub fn new(collection: Collection<SMSVerificationCode>) -> Self {
        Self { collection }
    }
}

impl Store for MongoStore {
    async fn get(&self, phone: &str) -> Result<Option<SMSVerificationCode>, Box<dyn Display>> {
        self.collection
            .find_one(doc! {"phone": phone}, None)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Display>)
    }
    async fn put(&self, phone: &str, code: &str) -> Result<(), Box<dyn Display>> {
        self.collection
            .update_one(
                doc! {"phone": phone},
                doc! {"$set": {"code": code, "sent_at": Utc::now().timestamp()}},
                UpdateOptions::builder().upsert(true).build(),
            )
            .await
            .map_or_else(|e| Err(Box::new(e) as Box<dyn Display>), |_| Ok(()))
    }
}
