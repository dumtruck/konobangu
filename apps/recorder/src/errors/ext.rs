pub trait RAnyhowResultExt<T>: snafu::ResultExt<T, anyhow::Error> {
    fn to_dyn_boxed(self) -> Result<T, Box<dyn std::error::Error + Send + Sync>>;
}

impl<T> RAnyhowResultExt<T> for Result<T, anyhow::Error> {
    fn to_dyn_boxed(self) -> Result<T, Box<dyn std::error::Error + Send + Sync>> {
        self.map_err(|e| e.into())
    }
}
