use crate::net::Shared;
use std::sync::OnceLock;

pub fn frames(rally: bool) -> Shared {
    static LOBBY: OnceLock<Shared> = OnceLock::new();
    static RALLY: OnceLock<Shared> = OnceLock::new();
    let cache = if rally { &RALLY } else { &LOBBY };

    cache
        .get_or_init(|| {
            #[cfg(feature = "prebaked")]
            {
                let data: &[u8] = if rally {
                    include_bytes!(concat!(env!("OUT_DIR"), "/rally.bin"))
                } else {
                    include_bytes!(concat!(env!("OUT_DIR"), "/lobby.bin"))
                };
                Shared::from(data)
            }

            #[cfg(not(feature = "prebaked"))]
            {
                let t0 = std::time::Instant::now();
                let buf = crate::mapgen::build_world_frames(rally);
                println!(
                    "[CPU MapGen] rally={rally} · {} KB · {:.1} ms",
                    buf.len() / 1024,
                    t0.elapsed().as_secs_f64() * 1000.0
                );
                Shared::from(buf)
            }
        })
        .clone()
}
