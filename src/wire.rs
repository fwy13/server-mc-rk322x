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
