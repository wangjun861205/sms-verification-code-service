use std::fmt::Display;

pub trait Sender {
    async fn send(&self, phone: &str, code: &str) -> Result<(), Box<dyn Display>>;
}
