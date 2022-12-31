pub mod cmd;
pub mod config;
pub mod id;
pub mod logger;
pub mod version;

#[cfg(feature = "debug")]
pub mod tracing;

pub fn init_tracing() {
    #[cfg(feature = "debug")]
    crate::tracing::init_tracing();
}
