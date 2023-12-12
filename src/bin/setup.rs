use error_stack::{Result, ResultExt};
use kyoka::{Config, SetupError, State};
use yansi::Paint;

async fn setup(cfg: Config) -> Result<(), SetupError> {
    let state = State::new(cfg).await.change_context(SetupError)?;
    kyoka::setup::cmd(&state).await?;

    Ok(())
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<(), SetupError> {
    let cfg = Config::from_env().change_context(SetupError)?;
    kyoka::setup::init_logging(&cfg)?;

    println!(
        "🔨 {} ({}) {}",
        "Setting up Kyoka".bold(),
        VERSION.bold(),
        "bot environment".bold(),
    );
    println!("This may take a while...");

    println!("> config: {:?}", cfg.dim().bold());
    println!();

    let rt = tokio::runtime::Builder::new_multi_thread()
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
