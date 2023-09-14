use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct AppConfig {
    pub example: u32,
}
