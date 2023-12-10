#[cfg(all(not(feature = "log_using_tracing"), feature = "log_using_log"))]
pub use log::error;
#[cfg(feature = "log_using_tracing")]
pub use tracing::error;

#[cfg(not(any(feature = "log_using_log", feature = "log_using_tracing")))]
macro_rules! error {
    ($msg:literal) => {};
}

#[cfg(not(any(feature = "log_using_log", feature = "log_using_tracing")))]
pub(crate) use error;
