use error_stack::{Result, ResultExt};
use kyoka_shard::{App, SetupError};

async fn runner() -> Result<(), SetupError> {
    let app = App::new()?;

    // Context will not be lost for both of them because
    // we're using error-stack to keep errors and its context.
    //
    // TODO: Implement graceful shutdown for these futures
    let service_result = tokio::try_join!(
        kyoka_shard::metrics::server::start(app.clone()),
        kyoka_shard::bot::start(app.clone()),
    );
    service_result?;

    Ok(())
}

fn main() -> Result<(), SetupError> {
    kyoka::util::init_logging().change_context(SetupError)?;

    // Any required events need to send by sentry will be
    // processed when `_sentry` variable is going to drop
    let _sentry = kyoka::sentry::init("kyoka-shard");
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(runner())
}
