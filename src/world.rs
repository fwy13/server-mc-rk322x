use crate::net::Shared;
use std::sync::OnceLock;

pub fn frames(rally: bool) -> Shared {
    static LOBBY: OnceLock<Shared> = OnceLock::new();
    static RALLY: OnceLock<Shared> = OnceLock::new();
    let (cache, data): (_, &[u8]) = if rally {
        (
            &RALLY,
            include_bytes!(concat!(env!("OUT_DIR"), "/rally.bin")),
        )
    } else {
        (
            &LOBBY,
            include_bytes!(concat!(env!("OUT_DIR"), "/lobby.bin")),
        )
    };
    cache.get_or_init(|| Shared::from(data)).clone()
}
