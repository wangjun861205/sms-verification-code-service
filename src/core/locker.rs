use std::fmt::Display;
pub trait Locker {
    async fn lock(&mut self, phone: &str) -> Result<(), Box<dyn Display>>;
    async fn unlock(&mut self, phone: &str) -> Result<(), Box<dyn Display>>;
}
