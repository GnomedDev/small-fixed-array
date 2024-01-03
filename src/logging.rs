#[cfg(all(not(feature = "log_using_tracing"), feature = "log_using_log"))]
pub use log::error;
#[cfg(feature = "log_using_tracing")]
pub use tracing::error;
