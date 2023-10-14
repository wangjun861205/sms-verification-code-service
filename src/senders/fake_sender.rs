use std::fmt::Display;

use crate::core::sender::Sender;

#[derive(Debug, Default, Clone)]
pub struct FakeSender;

impl Sender for FakeSender {
    async fn send(&self, phone: &str, code: &str) -> Result<(), Box<dyn Display>> {
        dbg!("phone: {}, code: {}", phone, code);
        Ok(())
    }
}
