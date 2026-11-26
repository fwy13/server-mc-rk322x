mod arena;
mod colors;
mod course;
mod games;
mod layout;
#[cfg(not(feature = "prebaked"))]
mod mapgen;
#[allow(dead_code)]
mod materials;
mod menu;
mod net;
mod wire;
mod world;
use games::{Game, GAMES};
use std::{
    env, io,
    net::{Ipv4Addr, SocketAddrV4, TcpListener},
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::sync_channel,
        Arc,
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
fn main() -> io::Result<()> {
    let mut addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 25565);
    let mut playlist = GAMES.to_vec();
    let mut max = 50usize;
    let mut stats = false;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--list" => {
                for g in GAMES {
                    let (k, n, r) = g.meta();
                    println!("{k:10} {n} — {r}")
                }
                return Ok(());
            }
            "--help" | "-h" => {
                println!("rk-festival --listen LAN_IP:25565 --game all|color,tnt,... --max-players 50 --stats\nrk-festival --list");
                return Ok(());
            }
            "--listen" => {
                addr = args
                    .next()
                    .and_then(|s| s.parse().ok())
                    .ok_or_else(wire::invalid)?
            }
            "--max-players" => {
                max = args
                    .next()
                    .and_then(|s| s.parse().ok())
                    .filter(|n| (1..=128).contains(n))
                    .ok_or_else(wire::invalid)?
            }
            "--stats" => stats = true,
            "--game" => {
                let s = args.next().ok_or_else(wire::invalid)?;
                if s != "all" {
                    playlist = s
                        .split(',')
                        .map(|k| {
                            GAMES.into_iter().find(|g| g.meta().0 == k).ok_or_else(|| {
                                io::Error::new(
                                    io::ErrorKind::InvalidInput,
                                    format!("Trò không có: {k}"),
                                )
                            })
                        })
                        .collect::<io::Result<Vec<Game>>>()?
                }
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Tham số không hợp lệ: {arg}"),
                ))
            }
        }
    }
    if !addr.ip().is_private() && !addr.ip().is_loopback() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Chọn IPv4 LAN hoặc loopback",
        ));
    }
    let listener = TcpListener::bind(addr)?;
    let (tx, rx) = sync_channel(256);
    let online = Arc::new(AtomicUsize::new(0));
    let count = online.clone();
    let slots = Arc::new(AtomicUsize::new(0));
    thread::spawn(move || {
        let mut seq = 0i32;
        for s in listener.incoming().flatten() {
            if slots.fetch_add(1, Ordering::Relaxed) >= max + 8 {
                slots.fetch_sub(1, Ordering::Relaxed);
                continue;
            }
            let slots = slots.clone();
            let tx = tx.clone();
            let count = count.clone();
            seq = seq.wrapping_add(1);
            let id = seq;
            let rescue = slots.clone();
            if thread::Builder::new()
                .stack_size(192 * 1024)
                .spawn(move || {
                    let _ = net::connect(s, id, tx, &count, max);
                    slots.fetch_sub(1, Ordering::Relaxed);
                })
                .is_err()
            {
                rescue.fetch_sub(1, Ordering::Relaxed);
            }
        }
    });
    println!(
        "RK Festival Java 26.3 · {addr} · {} trò · {max} chỗ",
        playlist.len()
    );
    let mut arena = arena::Arena::new(
        max,
        playlist,
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64,
    );
    let base = Instant::now();
    let mut next = base;
    let mut last = None;
    let mut sample = base;
    let mut gaps = Vec::with_capacity(210);
    loop {
        let now = Instant::now();
        for event in rx.try_iter().take(256) {
            arena.event(event)
        }
        arena.tick(now.duration_since(base).as_millis() as u64);
        online.store(arena.online(), Ordering::Relaxed);
        if stats {
            if let Some(previous) = last {
                gaps.push(now.duration_since(previous).as_micros());
            }
            if now.duration_since(sample) >= Duration::from_secs(10) {
                gaps.sort_unstable();
                println!(
                    "online={} TPS={:.2} p95_gap_ms={:.1} max_gap_ms={:.1}",
                    arena.online(),
                    gaps.len() as f64 / now.duration_since(sample).as_secs_f64(),
                    gaps[(gaps.len() * 95).div_ceil(100) - 1] as f64 / 1000.,
                    gaps[gaps.len() - 1] as f64 / 1000.
                );
                gaps.clear();
                sample = now
            }
        }
        last = Some(now);
        next += Duration::from_millis(50);
        if next < Instant::now() {
            next = Instant::now()
        }
        thread::sleep(next.saturating_duration_since(Instant::now()));
    }
}
