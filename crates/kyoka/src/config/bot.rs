use crate::util::{env, Sensitive};
use error_stack::{Result, ResultExt};

use super::LoadError;

#[derive(Debug)]
pub struct Bot {
    gateway_proxy_url: Option<String>,
    proxy_url: Option<String>,
    proxy_use_http: bool,
    // TODO: too wordy, make it simpler
    reload_commands_on_start: bool,
    token: Sensitive<String>,
}

impl Bot {
    #[track_caller]
    pub fn from_env() -> Result<Self, LoadError> {
        let token =
            env::required_var("DISCORD_BOT_TOKEN").change_context(LoadError)?;

        let reload_commands_on_start =
            env::var_parse("RELOAD_COMMANDS_ON_START")
                .change_context(LoadError)?
                .unwrap_or(false);

        let gateway_proxy_url =
            env::var("BOT_GATEWAY_PROXY_URL").change_context(LoadError)?;

        let proxy_url = env::var("BOT_PROXY_URL").change_context(LoadError)?;
        let proxy_use_http = match &proxy_url {
            Some(..) => env::var_parse("BOT_PROXY_USE_HTTP")
                .change_context(LoadError)?
                .unwrap_or(false),
            None => false,
        };

        Ok(Self {
            gateway_proxy_url,
            proxy_url,
            proxy_use_http,
            reload_commands_on_start,
            token: token.into(),
        })
    }
}

impl Bot {
    /// Reloads all required commands upon startup
    #[must_use]
    pub const fn reload_commands_on_start(&self) -> bool {
        self.reload_commands_on_start
    }

    #[must_use]
    pub fn gateway_proxy_url(&self) -> Option<&str> {
        self.gateway_proxy_url.as_deref()
    }

    #[must_use]
    pub fn proxy_url(&self) -> Option<&str> {
        self.proxy_url.as_deref()
    }

    #[must_use]
    pub const fn proxy_use_http(&self) -> bool {
        self.proxy_use_http
    }

    /// Gets the Discord bot token for a configured bot
    #[must_use]
    pub fn token(&self) -> &str {
        &self.token
    }
}
