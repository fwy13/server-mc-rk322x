use crate::wire::*;
use std::{
    io::{self, BufReader, Read, Write},
    net::{Shutdown, TcpStream},
    sync::{
        mpsc::{sync_channel, SyncSender},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};
pub type Shared = Arc<[u8]>;
pub struct Inbox {
    pub pos: Pos,
    pub punch: bool,
    pub dirty: bool,
    pub ready: bool,
    pub ack: i32,
}
pub struct Guest {
    pub name: String,
    pub uuid: [u8; 16],
    pub out: SyncSender<Shared>,
    pub input: Arc<Mutex<Inbox>>,
    pub socket: TcpStream,
}
pub enum Event {
    Join(i32, Guest),
    Leave(i32),
    Chat(i32, String),
    MenuClick(i32, i16),
    MenuClose(i32),
}
pub const LOBBY: Pos = Pos {
    x: 21.5,
    y: 78.,
    z: 0.5,
    yaw: 90.,
    pitch: 12.,
};
pub fn connect(
    mut s: TcpStream,
    id: i32,
    events: SyncSender<Event>,
    online: &std::sync::atomic::AtomicUsize,
    max: usize,
) -> io::Result<()> {
    s.set_read_timeout(Some(Duration::from_secs(20)))?;
    s.set_write_timeout(Some(Duration::from_secs(8)))?;
    s.set_nodelay(true)?;
    let mut r = BufReader::new(s.try_clone()?);
    let (pid, b) = read_packet(&mut r)?;
    let mut h = &b[..];
    if pid != 0 {
        return Err(invalid());
    }
    let ver = int(&mut h)?;
    text(&mut h, 1024)?;
    take(&mut h, 2)?;
    let state = int(&mut h)?;
    if state == 1 {
        if read_packet(&mut r)?.0 != 0 {
            return Err(invalid());
        }
        let n = online.load(std::sync::atomic::Ordering::Relaxed);
        let status = format!(
            r#"{{"version":{{"name":"26.3","protocol":777}},"players":{{"max":{max},"online":{n}}},"description":{{"text":"RK Festival · Rust · LAN"}}}}"#
        );
        s.write_all(&packet(0, &string(&status)))?;
        let (i, p) = read_packet(&mut r)?;
        if i == 1 && p.len() == 8 {
            s.write_all(&packet(1, &p))?
        }
        return Ok(());
    }
    if state != 2 || ver != 777 {
        return Err(invalid());
    }
    let (i, b) = read_packet(&mut r)?;
    let mut h = &b[..];
    let name = text(&mut h, 16)?;
    if i != 0
        || name.is_empty()
        || !name.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
        || h.len() != 16
    {
        return Err(invalid());
    }
    let mut uuid = [0; 16];
    std::fs::File::open("/dev/urandom")?.read_exact(&mut uuid)?;
    uuid[6] = (uuid[6] & 15) | 64;
    uuid[8] = (uuid[8] & 63) | 128;
    let mut b = uuid.to_vec();
    b.extend(string(&name));
    b.push(0);
    b.extend(uuid);
    s.write_all(&packet(2, &b))?;
    if read_packet(&mut r)?.0 != 3 {
        return Err(invalid());
    }
    let known = include_bytes!("../assets/known.bin");
    s.write_all(&packet(13, include_bytes!("../assets/features.bin")))?;
    s.write_all(&packet(15, known))?;
    loop {
        let (i, b) = read_packet(&mut r)?;
        if i == 7 {
            if b != known {
                return Err(invalid());
            }
            break;
        }
    }
    s.write_all(include_bytes!("../assets/config.frames"))?;
    loop {
        if read_packet(&mut r)?.0 == 3 {
            break;
        }
    }
    let (mut writer, control) = (s.try_clone()?, s.try_clone()?);
    let (tx, rx) = sync_channel::<Shared>(128);
    thread::Builder::new()
        .stack_size(128 * 1024)
        .spawn(move || {
            while let Ok(data) = rx.recv() {
                if writer.write_all(&data).is_err() {
                    break;
                }
            }
            let _ = writer.shutdown(Shutdown::Both);
        })?;
    let input = Arc::new(Mutex::new(Inbox {
        pos: LOBBY,
        punch: false,
        dirty: false,
        ready: false,
        ack: -1,
    }));
    events
        .send(Event::Join(
            id,
            Guest {
                name,
                uuid,
                out: tx,
                input: input.clone(),
                socket: control,
            },
        ))
        .map_err(|_| invalid())?;
    let result = (|| {
        s.set_read_timeout(Some(Duration::from_secs(45)))?;
        let mut last_chat = None;
        loop {
            let (i, b) = read_packet(&mut r)?;
            let mut data = &b[..];
            match i {
                0x09 => {
                    let raw = text(&mut data, 1024)?;
                    if raw.encode_utf16().count() > 256 {
                        return Err(invalid());
                    }
                    let message: String = raw
                        .trim()
                        .chars()
                        .filter(|c| !c.is_control() && *c != '§')
                        .collect();
                    let now = std::time::Instant::now();
                    if !message.trim().is_empty()
                        && last_chat.is_none_or(|prev| {
                            now.duration_since(prev) >= Duration::from_millis(500)
                        })
                    {
                        let _ = events.try_send(Event::Chat(id, message));
                        last_chat = Some(now);
                    }
                }
                0x10 => {
                    let win_id = int(&mut data)?;
                    let _state_id = int(&mut data)?;
                    let slot = i16::from_be_bytes(take(&mut data, 2)?.try_into().unwrap());
                    if win_id == crate::menu::WINDOW_ID as i32 {
                        let _ = events.try_send(Event::MenuClick(id, slot));
                    }
                }
                0x11 => {
                    let _ = events.try_send(Event::MenuClose(id));
                }
                0 => {
                    let k = int(&mut data)?;
                    if data.len() == 32 {
                        input.lock().unwrap().ack = k;
                    }
                }
                0x2e | 0x29 | 0x42 | 0x43 => input.lock().unwrap().punch = true,
                0x2c => input.lock().unwrap().ready = true,
                0x1e | 0x1f | 0x20 => {
                    let expected = match i {
                        0x1e => 25,
                        0x1f => 33,
                        _ => 9,
                    };
                    if b.len() != expected {
                        return Err(invalid());
                    }
                    let mut slot = input.lock().unwrap();
                    let mut p = slot.pos;
                    if i != 0x20 {
                        p.x = f64::from_be_bytes(take(&mut data, 8)?.try_into().unwrap());
                        p.y = f64::from_be_bytes(take(&mut data, 8)?.try_into().unwrap());
                        p.z = f64::from_be_bytes(take(&mut data, 8)?.try_into().unwrap())
                    }
                    if i != 0x1e {
                        p.yaw = f32::from_be_bytes(take(&mut data, 4)?.try_into().unwrap());
                        p.pitch = f32::from_be_bytes(take(&mut data, 4)?.try_into().unwrap())
                    }
                    if ![p.x, p.y, p.z, p.yaw as f64, p.pitch as f64]
                        .iter()
                        .all(|x| x.is_finite())
                    {
                        return Err(invalid());
                    }
                    slot.pos = p;
                    slot.dirty = true;
                }
                _ => (),
            }
        }
    })();
    let _ = events.send(Event::Leave(id));
    result
}
