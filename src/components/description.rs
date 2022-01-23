use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Description {
    pub name: String,
    pub description: String,
}

impl Default for Description {
    fn default() -> Self {
        Self {
            name: String::from("?"),
            description: String::new(),
        }
    }
}
