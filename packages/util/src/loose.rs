use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BooleanLike {
    Boolean(bool),
    String(String),
    Number(i32),
}

impl BooleanLike {
    pub fn as_bool(&self) -> bool {
        match self {
            BooleanLike::Boolean(b) => *b,
            BooleanLike::String(s) => s.to_lowercase() == "true",
            BooleanLike::Number(n) => *n != 0,
        }
    }
}
