// use std::num::NonZeroU64;

use std::net::{IpAddr, Ipv4Addr};

pub fn proxy_use_http() -> bool {
    false
}

pub fn reload_commands_on_start() -> bool {
    false
}

pub fn metrics_host() -> IpAddr {
    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
}

pub fn metrics_port() -> u16 {
    5000
}

// pub fn shard_id() -> u64 {
//     0
// }

// pub fn shard_cfg_amount() -> NonZeroU64 {
//     NonZeroU64::new(1).unwrap()
// }
