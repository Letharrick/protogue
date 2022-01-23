use serde::Deserialize;

#[derive(Debug, Deserialize, Copy, Clone)]
pub struct Weight {
    pub grams: u32,
}

impl Default for Weight {
    fn default() -> Self {
        Self { grams: 1 }
    }
}
