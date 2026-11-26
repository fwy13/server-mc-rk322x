use crate::{
    colors, layout, materials,
    wire::{int, packet, take, vi},
};
use std::collections::BTreeMap;

fn number(s: &mut &[u8]) -> usize {
    int(s).unwrap().try_into().unwrap()
}
fn bytes<'a>(s: &mut &'a [u8], n: usize) -> &'a [u8] {
    take(s, n).unwrap()
}
fn unpack(words: &[u8], bits: usize, count: usize) -> Vec<i32> {
    let width = 64 / bits;
    (0..count)
        .map(|i| {
            let start = i / width * 8;
            ((u64::from_be_bytes(words[start..start + 8].try_into().unwrap())
                >> (i % width * bits))
                & ((1 << bits) - 1)) as i32
        })
        .collect()
}
fn pack(values: &[i32], bits: usize) -> Vec<u8> {
    values
        .chunks(64 / bits)
        .flat_map(|row| {
            row.iter()
                .enumerate()
                .fold(0u64, |word, (i, v)| word | ((*v as u64) << (i * bits)))
                .to_be_bytes()
        })
        .collect()
}
fn palette(s: &mut &[u8], count: usize) -> Vec<i32> {
    let bits = bytes(s, 1)[0] as usize;
    if bits == 0 {
        return vec![number(s) as i32; count];
    }
    let entries = number(s);
    let table: Vec<i32> = (0..entries).map(|_| number(s) as i32).collect();
    unpack(bytes(s, count.div_ceil(64 / bits) * 8), bits, count)
        .iter()
        .map(|&i| table[i as usize])
        .collect()
}
fn write_palette(out: &mut Vec<u8>, values: &[i32], min: usize) {
    let mut table = values.to_vec();
    table.sort_unstable();
    table.dedup();
    if table.len() == 1 {
        out.push(0);
        out.extend(vi(table[0]));
        return;
    }
    let bits = (usize::BITS - (table.len() - 1).leading_zeros()) as usize;
    let bits = bits.max(min);
    assert!(bits <= 8);
    out.push(bits as u8);
    out.extend(vi(table.len() as i32));
    for &v in &table {
        out.extend(vi(v))
    }
    let encoded: Vec<_> = values
        .iter()
        .map(|v| table.binary_search(v).unwrap() as i32)
        .collect();
    out.extend(pack(&encoded, bits))
}
type Edits = BTreeMap<(i32, i32), Vec<(i32, i32, i32, i32)>>;
fn chunk(x: i32, z: i32, template: &[u8], edits: &[(i32, i32, i32, i32)], race: bool) -> Vec<u8> {
    let mut s = template;
    let mut heights = vec![];
    for _ in 0..number(&mut s) {
        let id = number(&mut s);
        let n = number(&mut s);
        bytes(&mut s, n * 8);
        heights.push(id);
    }
    let size = number(&mut s);
    let mut sections = bytes(&mut s, size);
    let mut data = vec![];
    let mut surface = [0; 256];
    let mut floor = [0; 256];
    for section in 0..24 {
        let bottom = section * 16 - 64;
        bytes(&mut sections, 4);
        let mut states = palette(&mut sections, 4096);
        let biomes = palette(&mut sections, 64);
        for &(bx, y, bz, v) in edits {
            if y >= bottom && y < bottom + 16 {
                states[(((y & 15) << 8) | ((bz & 15) << 4) | (bx & 15)) as usize] = v
            }
        }
        for (i, &v) in states.iter().enumerate().filter(|(_, v)| **v != 0) {
            let h = bottom + ((i >> 8) as i32) + 65;
            surface[i & 255] = h;
            if v != materials::WATER {
                floor[i & 255] = h
            }
        }
        data.extend((states.iter().filter(|&&v| v != 0).count() as u16).to_be_bytes());
        data.extend(
            (states.iter().filter(|&&v| v == materials::WATER).count() as u16).to_be_bytes(),
        );
        write_palette(&mut data, &states, 4);
        write_palette(&mut data, &biomes, 1);
    }
    assert!(sections.is_empty());
    let mut body = x.to_be_bytes().to_vec();
    body.extend(z.to_be_bytes());
    body.extend(vi(heights.len() as i32));
    for id in heights {
        let v = pack(if id == 3 || id == 2 { &floor } else { &surface }, 9);
        body.extend(vi(id as i32));
        body.extend(vi((v.len() / 8) as i32));
        body.extend(v);
    }
    body.extend(vi(data.len() as i32));
    body.extend(data);
    if race {
        assert_eq!(number(&mut s), 0);
        body.push(0);
        // Captured terrain sits below the course. Extend the skylight masks to
        // cover its two occupied sections plus their immediate neighbours.
        let masks: Vec<Vec<u8>> = (0..4)
            .map(|_| {
                let n = number(&mut s);
                bytes(&mut s, n).to_vec()
            })
            .collect();
        let mut sky = BTreeMap::new();
        let count = number(&mut s);
        for idx in 0..masks[0].len() * 8 {
            if masks[0][idx / 8] & (1 << (idx % 8)) != 0 {
                let n = number(&mut s);
                sky.insert(idx, bytes(&mut s, n).to_vec());
            }
        }
        assert_eq!(sky.len(), count);
        let mut masks = masks;
        for idx in 9..=12 {
            if !sky.contains_key(&idx) {
                sky.insert(idx, vec![255; 2048]);
                if masks[0].len() < 2 {
                    masks[0].resize(2, 0);
                }
                masks[0][idx / 8] |= 1 << (idx % 8);
                if let Some(v) = masks[2].get_mut(idx / 8) {
                    *v &= !(1 << (idx % 8));
                }
            }
        }
        for m in masks {
            body.extend(vi(m.len() as i32));
            body.extend(m);
        }
        body.extend(vi(sky.len() as i32));
        for v in sky.into_values() {
            body.extend(vi(v.len() as i32));
            body.extend(v);
        }
    }
    body.extend(s);
    packet(46, &body)
}

pub fn build_world_frames(race: bool) -> Vec<u8> {
    let mut src = &include_bytes!("../assets/world.frames")[..];
    let mut templates = BTreeMap::new();
    while !src.is_empty() {
        let n = number(&mut src);
        let mut frame = bytes(&mut src, n);
        if number(&mut frame) == 46 {
            let x = i32::from_be_bytes(bytes(&mut frame, 4).try_into().unwrap());
            let z = i32::from_be_bytes(bytes(&mut frame, 4).try_into().unwrap());
            templates.insert((x, z), frame);
        }
    }
    let mut edits = Edits::new();
    for x in -9..9 {
        for z in -9..9 {
            for (y, v) in [(72, colors::WHITE), (66, 0)] {
                edits
                    .entry((x >> 4, z >> 4))
                    .or_default()
                    .push((x, y, z, v));
            }
        }
    }
    if race {
        for ((x, y, z), v) in layout::landscape() {
            edits
                .entry((x >> 4, z >> 4))
                .or_default()
                .push((x, y, z, v));
        }
    }
    let mut out = packet(12, &[]);
    let mut count = 0;
    for x in -3..=4 {
        for z in -3..=3 {
            let t = templates.get(&(x, z)).unwrap_or(&templates[&(-2, -2)]);
            let e = edits.get(&(x, z)).map_or(&[][..], |v| v.as_slice());
            out.extend(chunk(x, z, t, e, race && e.iter().any(|p| p.1 >= 80)));
            count += 1;
        }
    }
    out.extend(packet(11, &vi(count)));
    out
}
