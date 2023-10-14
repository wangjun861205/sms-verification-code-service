use std::fmt::Display;

use super::entites::SMSVerificationCode;

pub trait Store {
    async fn get(&self, phone: &str) -> Result<Option<SMSVerificationCode>, Box<dyn Display>>;
    async fn put(&self, phone: &str, code: &str) -> Result<(), Box<dyn Display>>;
}
