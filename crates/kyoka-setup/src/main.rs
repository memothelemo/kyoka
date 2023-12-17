use error_stack::{Result, ResultExt};
use kyoka::config;
use kyoka::perform_request;
use yansi::Paint;

use thiserror::Error;

#[derive(Debug, Error)]
#[error("Failed to setup Kyoka environment")]
struct SetupError;

async fn setup(cfg: config::Bot) -> Result<(), SetupError> {
    let http = kyoka::util::make_http_client(&cfg);
    let app_info =
        perform_request!(http.current_user_application(), SetupError).await?;

    let interaction = http.interaction(app_info.id);
    kyoka::util::setup_cmds(interaction).await.change_context(SetupError)?;

    Ok(())
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<(), SetupError> {
    let cfg = config::Bot::from_env().change_context(SetupError)?;
    kyoka::util::init_logging().change_context(SetupError)?;

    println!(
        "🔨 {} v{} {}",
        "Setting up Kyoka".bold(),
        VERSION.bold(),
        "bot environment".bold(),
    );
    println!("This may take a while...");

    println!("> config: {:?}", cfg.dim().bold());
    println!();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to init tokio runtime");

    if let Err(err) = rt.block_on(setup(cfg)) {
        println!(
            "{}",
            "❌ Failed to setup bot environment!".bold().bright_red()
        );
        println!("Any changes to the database are not permanently saved (in transaction)");
        println!();
        println!("{err:?}");
        println!("{}", "⚠️ This may be a bug! Please file this issue at: https://github.com/memothelemo/kyoka/issues".bright_yellow().bold());
    } else {
        println!(
            "{}",
            "✅ Successfully initialized bot environment!"
                .bright_green()
                .bold()
        );
        println!("You may start the bot session/shard program now.");
    }

    Ok(())
}
