use std::fmt::Display;

#[derive(Debug)]
pub struct OptDynErr(Option<Box<dyn std::error::Error + Send + Sync>>);

impl AsRef<dyn snafu::Error> for OptDynErr {
    fn as_ref(&self) -> &(dyn snafu::Error + 'static) {
        self
    }
}

impl OptDynErr {
    pub fn some_boxed<E: std::error::Error + Send + Sync + 'static>(e: E) -> Self {
        Self(Some(Box::new(e)))
    }

    pub fn some(e: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self(Some(e))
    }

    pub fn none() -> Self {
        Self(None)
    }
}

impl Display for OptDynErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Some(e) => e.fmt(f),
            None => write!(f, "None"),
        }
    }
}

impl snafu::Error for OptDynErr {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl From<Option<Box<dyn std::error::Error + Send + Sync>>> for OptDynErr {
    fn from(value: Option<Box<dyn std::error::Error + Send + Sync>>) -> Self {
        Self(value)
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for OptDynErr {
    fn from(value: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::some(value)
    }
}

pub trait AnyhowResultExt<T>: snafu::ResultExt<T, anyhow::Error> {
    fn to_dyn_boxed(self) -> Result<T, Box<dyn std::error::Error + Send + Sync>>;
}

impl<T> AnyhowResultExt<T> for Result<T, anyhow::Error> {
    fn to_dyn_boxed(self) -> Result<T, Box<dyn std::error::Error + Send + Sync>> {
        self.map_err(|e| e.into())
    }
}
