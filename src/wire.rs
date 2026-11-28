use std::io::{self, Read};
pub type Buf = Vec<u8>;
pub fn vi(n: i32) -> Buf {
    let mut n = n as u32;
    let mut b = vec![];
    while n >= 128 {
        b.push(n as u8 | 128);
        n >>= 7;
    }
    b.push(n as u8);
    b
}
pub fn string(s: &str) -> Buf {
    [vi(s.len() as i32), s.as_bytes().to_vec()].concat()
}
pub fn invalid() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "invalid game packet")
}
pub fn int(r: &mut impl Read) -> io::Result<i32> {
    let mut v = 0u32;
    for shift in (0..35).step_by(7) {
        let mut b = [0];
        r.read_exact(&mut b)?;
        if shift == 28 && b[0] > 15 {
            return Err(invalid());
        }
        v |= ((b[0] & 127) as u32) << shift;
        if b[0] < 128 {
            return Ok(v as i32);
        }
    }
    Err(invalid())
}
pub fn take<'a>(r: &mut &'a [u8], n: usize) -> io::Result<&'a [u8]> {
    if n > r.len() {
        return Err(invalid());
    }
    let (a, b) = r.split_at(n);
    *r = b;
    Ok(a)
}
pub fn text(r: &mut &[u8], max: usize) -> io::Result<String> {
    let n = int(r)? as usize;
    if n > max {
        return Err(invalid());
    }
    String::from_utf8(take(r, n)?.to_vec()).map_err(|_| invalid())
}
pub fn read_packet(r: &mut impl Read) -> io::Result<(i32, Buf)> {
    let n = int(r)?;
    if !(1..=65536).contains(&n) {
        return Err(invalid());
    }
    let mut b = vec![0; n as usize];
    r.read_exact(&mut b)?;
    let mut tail = &b[..];
    let id = int(&mut tail)?;
    Ok((id, tail.to_vec()))
}
pub fn packet(id: i32, b: &[u8]) -> Buf {
    let i = vi(id);
    [vi((i.len() + b.len()) as i32).as_slice(), &i, b].concat()
}
fn short(s: &str) -> Buf {
    // NBT strings use Java's modified UTF-8, including surrogate pairs for emoji.
    let mut body = Vec::with_capacity(s.len() + 2);
    body.extend([0, 0]);
    for u in s.encode_utf16() {
        match u {
            1..=127 => body.push(u as u8),
            0..=2047 => body.extend([(0xc0 | (u >> 6)) as u8, (0x80 | (u & 63)) as u8]),
            _ => body.extend([
                (0xe0 | (u >> 12)) as u8,
                (0x80 | ((u >> 6) & 63)) as u8,
                (0x80 | (u & 63)) as u8,
            ]),
        }
    }
    let len = (body.len() - 2) as u16;
    body[..2].copy_from_slice(&len.to_be_bytes());
    body
}
pub fn component(s: &str, color: &str) -> Buf {
    let mut b = vec![10];
    for (k, v) in [("text", s), ("color", color)] {
        b.push(8);
        b.extend(short(k));
        b.extend(short(v))
    }
    b.push(0);
    b
}
pub fn chat(s: &str) -> Buf {
    let mut b = component(s, "white");
    b.push(0);
    packet(0x7c, &b)
}
pub fn title(s: &str, sub: &str, color: &str) -> Buf {
    let mut times = vec![];
    for t in [2i32, 32, 5] {
        times.extend(t.to_be_bytes())
    }
    [
        packet(0x75, &times),
        packet(0x72, &component(sub, "white")),
        packet(0x74, &component(s, color)),
    ]
    .concat()
}
pub fn board() -> Buf {
    let mut b = string("festival");
    b.push(0);
    b.extend(component("✦ RK FESTIVAL ✦", "gold"));
    b.extend([0, 1, 0]);
    let mut d = vec![1];
    d.extend(string("festival"));
    [packet(0x6c, &b), packet(0x64, &d)].concat()
}
pub fn row(i: usize, s: &str) -> Buf {
    let mut b = string(&format!("line{i}"));
    b.extend(string("festival"));
    b.extend(vi(5 - i as i32));
    b.push(1);
    b.extend(component(
        s,
        ["yellow", "aqua", "green", "gold", "white"][i],
    ));
    b.push(0);
    packet(0x70, &b)
}
pub fn block(x: i32, y: i32, z: i32, state: i32) -> Buf {
    let pos = ((x as u64 & 0x3ffffff) << 38) | ((z as u64 & 0x3ffffff) << 12) | (y as u64 & 4095);
    let mut b = pos.to_be_bytes().to_vec();
    b.extend(vi(state));
    packet(8, &b)
}
#[derive(Clone, Copy, Debug)]
pub struct Pos {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
}
impl Pos {
    pub fn at(x: f64, y: f64, z: f64) -> Self {
        Self {
            x,
            y,
            z,
            yaw: 0.,
            pitch: 0.,
        }
    }
    fn xyz(self) -> Buf {
        [
            self.x.to_be_bytes(),
            self.y.to_be_bytes(),
            self.z.to_be_bytes(),
        ]
        .concat()
    }
}
pub fn teleport(k: i32, p: Pos) -> Buf {
    let mut b = vi(k);
    b.extend(p.xyz());
    b.extend([0; 24]);
    b.extend(p.yaw.to_be_bytes());
    b.extend(p.pitch.to_be_bytes());
    b.extend([0; 4]);
    packet(0x49, &b)
}
pub fn info(id: &[u8; 16], name: &str) -> Buf {
    let mut b = vec![255, 1];
    b.extend(id);
    b.extend(string(name));
    b.extend([0, 0, 2, 1, 0, 0, 0, 0]);
    packet(0x47, &b)
}
pub fn spawn(id: i32, u: &[u8; 16], p: Pos) -> Buf {
    let mut b = vi(id);
    b.extend(u);
    b.extend(vi(159));
    b.extend(p.xyz());
    b.extend([0; 5]);
    packet(1, &b)
}
pub fn movement(id: i32, p: Pos) -> Buf {
    let mut b = vi(id);
    b.push(0);
    b.extend(p.xyz());
    b.extend(p.yaw.to_be_bytes());
    b.extend(p.pitch.to_be_bytes());
    b.push(1);
    packet(0x23, &b)
}

pub fn action_bar(s: &str, color: &str) -> Buf {
    packet(0x59, &component(s, color))
}
pub fn boss_bar_add(uuid: &[u8; 16], s: &str, color: &str, health: f32, bar_color: i32) -> Buf {
    let mut b = uuid.to_vec();
    b.extend(vi(0));
    b.extend(component(s, color));
    b.extend(health.clamp(0.0, 1.0).to_be_bytes());
    b.extend(vi(bar_color));
    b.extend(vi(0));
    b.push(0);
    packet(0x09, &b)
}
pub fn boss_bar_update(uuid: &[u8; 16], s: &str, color: &str, health: f32) -> Buf {
    let mut h = uuid.to_vec();
    h.extend(vi(2));
    h.extend(health.clamp(0.0, 1.0).to_be_bytes());
    let mut t = uuid.to_vec();
    t.extend(vi(3));
    t.extend(component(s, color));
    [packet(0x09, &h), packet(0x09, &t)].concat()
}
pub fn boss_bar_remove(uuid: &[u8; 16]) -> Buf {
    let mut b = uuid.to_vec();
    b.extend(vi(1));
    packet(0x09, &b)
}

pub fn transfer(host: &str, port: u16) -> Buf {
    let mut b = string(host);
    b.extend(vi(port as i32));
    packet(0x84, &b)
}
pub fn set_hotbar_item(slot_idx: i32, item_id: i32, count: i32) -> Buf {
    let mut b = vi(slot_idx);
    b.extend(vi(count));
    b.extend(vi(item_id));
    b.extend(vi(0));
    b.extend(vi(0));
    packet(0x6e, &b)
}
pub fn hologram_spawn(entity_id: i32, uuid: &[u8; 16], p: Pos, text: &str, color: &str) -> Buf {
    let mut add = vi(entity_id);
    add.extend(uuid);
    add.extend(vi(135)); // minecraft:text_display (26.3 protocol 777)
    add.extend(p.xyz());
    add.extend([0; 5]);

    let mut meta = vi(entity_id);
    // Index 15: billboard_render_constraints (BYTE=0) -> 3 = CENTER (luôn xoay về phía người chơi)
    meta.extend([15, 0, 3]);
    // Index 23: text (COMPONENT=5)
    meta.extend([23, 5]);
    meta.extend(component(text, color));
    // Index 25: background_color (VARINT=1) -> nền đen bán trong suốt ARGB 0x60000000
    meta.extend([25, 1]);
    meta.extend(vi(0x6000_0000u32 as i32));
    // Index 27: style_flags (BYTE=0) -> 1 = có bóng chữ (drop shadow)
    meta.extend([27, 0, 1]);
    meta.push(0xff);

    [packet(1, &add), packet(0x65, &meta)].concat()
}
pub fn remove_entities(ids: &[i32]) -> Buf {
    let mut b = vi(ids.len() as i32);
    for &id in ids {
        b.extend(vi(id));
    }
    packet(0x4e, &b)
}

pub fn sound(name: &str, p: Pos, volume: f32, pitch: f32) -> Buf {
    let mut b = vi(0); // Inline SoundEvent (không lệ thuộc registry ID)
    b.extend(string(name));
    b.push(0); // has_fixed_range = false
    b.extend(vi(0)); // SoundSource::MASTER
    b.extend(((p.x * 8.0) as i32).to_be_bytes());
    b.extend(((p.y * 8.0) as i32).to_be_bytes());
    b.extend(((p.z * 8.0) as i32).to_be_bytes());
    b.extend(volume.to_be_bytes());
    b.extend(pitch.to_be_bytes());
    b.extend(0i64.to_be_bytes());
    packet(0x77, &b)
}
pub fn tab_list(header: &str, footer: &str) -> Buf {
    let mut b = component(header, "gold");
    b.extend(component(footer, "aqua"));
    packet(0x7d, &b)
}
pub fn commands_tree(cmds: &[&str]) -> Buf {
    let mut b = vi((1 + cmds.len()) as i32);
    b.push(0x00); // root
    b.extend(vi(cmds.len() as i32));
    for i in 1..=cmds.len() {
        b.extend(vi(i as i32));
    }
    for &cmd in cmds {
        b.push(0x05); // literal | executable
        b.extend(vi(0));
        b.extend(string(cmd));
    }
    b.extend(vi(0));
    packet(0x10, &b)
}
pub fn hologram_text(entity_id: i32, text: &str, color: &str) -> Buf {
    let mut meta = vi(entity_id);
    meta.extend([23, 5]); // Index 23: text (COMPONENT=5)
    meta.extend(component(text, color));
    meta.push(0xff);
    packet(0x65, &meta)
}
pub fn map_item_data(map_id: i32, colors_16384: &[u8]) -> Buf {
    let mut b = vi(map_id);
    b.push(0); // scale = 0
    b.push(1); // locked = true
    b.push(0); // has_icons = false
    b.push(128); // columns
    b.push(128); // rows
    b.push(0); // x_offset
    b.push(0); // z_offset
    b.extend(vi(colors_16384.len() as i32));
    b.extend(colors_16384);
    packet(0x34, &b)
}

pub fn poster_frame(entity_id: i32, uuid: &[u8; 16], p: Pos, facing: i32, map_id: i32) -> Buf {
    let mut add = vi(entity_id);
    add.extend(uuid);
    add.extend(vi(61)); // minecraft:glow_item_frame (26.3)
    add.extend(p.xyz());
    add.extend([0, 0, 0, 0]); // velocity(0), pitch(0), yaw(0), head_yaw(0)
    add.extend(vi(facing)); // 2=North, 3=South, 4=West, 5=East

    let mut meta = vi(entity_id);
    // Index 9: ItemFrame DATA_ITEM (type SLOT = 7)
    // (Nếu vào game thấy khung trống, đổi số 9 thành 8)
    meta.extend([9, 7]);
    meta.extend(vi(1)); // count = 1
    meta.extend(vi(1238)); // minecraft:filled_map
    meta.extend(vi(1)); // components_to_add = 1
    meta.extend(vi(0)); // components_to_remove = 0
    meta.extend(vi(48)); // minecraft:map_id
    meta.extend(vi(map_id));
    meta.push(0xff);

    [packet(1, &add), packet(0x65, &meta)].concat()
}
pub fn resource_pack_push(
    uuid: &[u8; 16],
    url: &str,
    sha1: &str,
    forced: bool,
    prompt: Option<&str>,
) -> Buf {
    let mut b = uuid.to_vec();
    b.extend(string(url));
    b.extend(string(sha1));
    b.push(u8::from(forced));
    match prompt {
        Some(msg) => {
            b.push(1);
            b.extend(component(msg, "gold"));
        }
        None => b.push(0),
    }
    packet(0x52, &b)
}
pub fn poster_hd_spawn(entity_id: i32, uuid: &[u8; 16], p: Pos) -> Buf {
    let mut add = vi(entity_id);
    add.extend(uuid);
    add.extend(vi(135)); // minecraft:text_display (26.3 protocol 777)
    add.extend(p.xyz());
    add.extend([0; 5]); // yaw=0 -> quay mặt hướng Nam (+Z) về phía Đền Gacha

    let mut meta = vi(entity_id);
    meta.extend([15, 0, 0]); // Index 15: billboard = 0 (FIXED)
    meta.extend([23, 5]); // Index 23: text = ký tự ảnh HD \uE001 trong Resource Pack
    meta.extend(component("\u{E001}", "white"));
    meta.extend([25, 1]); // Index 25: background_color = 0 (trong suốt)
    meta.extend(vi(0));
    meta.extend([27, 0, 0]); // Index 27: style_flags = 0 (không bóng chữ)
    meta.push(0xff);
    [packet(1, &add), packet(0x65, &meta)].concat()
}
