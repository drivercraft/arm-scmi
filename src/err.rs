#[derive(thiserror::Error, Debug, Clone)]
pub enum ScmiError {
    #[error("Not supported")]
    NotSupported,
}
