pub mod cmd;
pub mod config;
pub mod metrics;
pub mod sentry;
pub mod util;

#[cfg(wasm)]
compile_error!("Kyoka cannot be compiled with WebAssembly.");
