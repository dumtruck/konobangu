pub enum Enviornment {
    Development,
    Production,
    Testing,
}

impl Enviornment {
    pub fn full_name(&self) -> &'static str {
        match &self {
            Self::Development => "development",
            Self::Production => "production",
            Self::Testing => "testing",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match &self {
            Self::Development => "dev",
            Self::Production => "prod",
            Self::Testing => "test",
        }
    }
}
