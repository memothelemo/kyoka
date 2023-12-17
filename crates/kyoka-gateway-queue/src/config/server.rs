use error_stack::{Result, ResultExt};
use kyoka::config::LoadError;
use kyoka::util::{env, Sensitive};
use std::net::{IpAddr, Ipv4Addr};

#[derive(Debug)]
pub struct Server {
    host: IpAddr,
    port: u16,
    token: Option<Sensitive<String>>,
    proxy_url: Option<String>,
    proxy_use_http: bool,
}

const DEFAULT_PORT: u16 = 3421;
const DEFAULT_HOST: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

impl Server {
    #[track_caller]
    pub fn from_env() -> Result<Self, LoadError> {
        let host = env::var_parse("GATEWAY_QUEUE_HOST")
            .change_context(LoadError)?
            .unwrap_or(DEFAULT_HOST);

        let port = env::var_parse("GATEWAY_QUEUE_PORT")
            .change_context(LoadError)?
            .unwrap_or(DEFAULT_PORT);

        let token = env::var("DISCORD_BOT_TOKEN")
            .change_context(LoadError)?
            .map(Sensitive::new);

        let proxy_url = env::var("BOT_PROXY_URL").change_context(LoadError)?;
        let proxy_use_http = match &proxy_url {
            Some(..) => env::var_parse("BOT_PROXY_USE_HTTP")
                .change_context(LoadError)?
                .unwrap_or(false),
            None => false,
        };

        Ok(Self { host, port, token, proxy_url, proxy_use_http })
    }
}

impl Server {
    #[must_use]
    pub const fn host(&self) -> IpAddr {
        self.host
    }

    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }

    #[must_use]
    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    #[must_use]
    pub fn proxy_url(&self) -> Option<&str> {
        self.proxy_url.as_deref()
    }

    #[must_use]
    pub const fn proxy_use_http(&self) -> bool {
        self.proxy_use_http
    }
}
