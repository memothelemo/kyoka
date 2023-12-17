pub mod config;
pub mod sentry;
pub mod util;

#[cfg(wasm)]
compile_error!("Kyoka cannot be compiled with WebAssembly.");
