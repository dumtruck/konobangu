use std::fmt::Display;

#[derive(Debug)]
pub struct OptionWhateverAsync(Option<Box<dyn std::error::Error + Send + Sync>>);

impl AsRef<dyn snafu::Error> for OptionWhateverAsync {
    fn as_ref(&self) -> &(dyn snafu::Error + 'static) {
        self
    }
}

impl OptionWhateverAsync {
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

impl Display for OptionWhateverAsync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Some(e) => e.fmt(f),
            None => write!(f, "None"),
        }
    }
}

impl snafu::Error for OptionWhateverAsync {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl From<Option<Box<dyn std::error::Error + Send + Sync>>> for OptionWhateverAsync {
    fn from(value: Option<Box<dyn std::error::Error + Send + Sync>>) -> Self {
        Self(value)
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for OptionWhateverAsync {
    fn from(value: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::some(value)
    }
}
