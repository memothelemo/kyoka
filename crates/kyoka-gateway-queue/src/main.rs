use error_stack::{Result, ResultExt};
use kyoka_gateway_queue::{config, SetupError};

fn main() -> Result<(), SetupError> {
    kyoka::util::init_logging().change_context(SetupError)?;

    let cfg = config::Server::from_env().change_context(SetupError)?;
    let _sentry = kyoka::sentry::init("kyoka-gateway-queue");

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(kyoka_gateway_queue::server::run(cfg))
}
