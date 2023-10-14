use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SMSVerificationCode {
    pub phone: String,
    pub code: String,
    pub sent_at: i64,
}
