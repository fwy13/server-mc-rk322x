#![allow(dead_code)]
use crate::colors::{BLUE, GOLD, LIME, RED, WHITE};
use crate::course::{GATES, START};
use std::collections::BTreeMap;
type Blocks = BTreeMap<(i32, i32, i32), i32>;
use crate::materials::*;

pub fn landscape() -> Blocks {
    let mut b = Blocks::new();
    for x in -30i32..=49 {
        for z in -30i32..=36 {
            let material = if x < -4 {
                GRASS
            } else if x < 21 {
                WATER
            } else if z < 4 {
                STONE
            } else {
                SNOW
            };
            b.insert((x, if material == WATER { 86 } else { 88 }, z), material);
        }
    }
    // Evergreen silhouettes beside the first leg; layered boughs leave the route open.
    for (x, z, h) in [
        (-28, -23, 7),
        (-15, -18, 6),
        (-28, -10, 6),
        (-16, -3, 8),
        (-28, 4, 7),
        (-15, 9, 6),
        (-28, 19, 8),
        (-11, 22, 7),
        (-24, 30, 6),
        (-11, -26, 8),
    ] {
        for y in 89..89 + h {
            b.insert((x, y, z), LOG);
        }
        for (dy, r) in [(2, 2i32), (3, 2), (4, 1), (5, 1), (6, 0)] {
            for dx in -r..=r {
                for dz in -r..=r {
                    if dx.abs() + dz.abs() <= r + 1 {
                        b.insert((x + dx, 89 + dy + (h - 6), z + dz), LEAVES);
                    }
                }
            }
        }
    }
    // Open watchtower with corner columns, a stepped roof and a lit beacon.
    for (cx, cz) in [(30, -7), (46, -23)] {
        for dx in -2i32..=2 {
            for dz in -2i32..=2 {
                for y in 89..=96 {
                    if y == 89 || y == 95 || (dx.abs() == 2 && dz.abs() == 2) {
                        b.insert((cx + dx, y, cz + dz), BRICKS);
                    }
                }
                b.insert((cx + dx, 97 - dx.abs().max(dz.abs()), cz + dz), RED);
            }
        }
        b.insert((cx, 98, cz), LIGHT);
    }
    // Snow spires and a boat with a striped sail.
    for (cx, cz, h) in [(29, 8, 7), (47, 14, 6), (30, 21, 9), (45, 32, 5)] {
        for dy in 0..h {
            let r = (h - dy) / 3;
            for dx in -r..=r {
                for dz in -r..=r {
                    if i32::abs(dx) + i32::abs(dz) <= r {
                        b.insert(
                            (cx + dx, 89 + dy, cz + dz),
                            if dy == h - 1 { SNOW } else { ICE },
                        );
                    }
                }
            }
        }
    }
    for z in -3i32..=3 {
        let w = if z.abs() == 3 { 0 } else { 1 };
        for x in -w..=w {
            b.insert((x, 87, z), PLANK);
        }
    }
    for y in 88..=94 {
        b.insert((0, y, 0), LOG);
    }
    for y in 90..94 {
        for z in 1..=94 - y {
            b.insert((0, y, z), if y % 2 == 0 { WHITE } else { RED });
        }
    }
    // A five-block path with gold center dashes. Each gateway faces the incoming leg.
    let mut from = START;
    for (idx, &(x, z, _)) in GATES.iter().enumerate() {
        let dx = (x - from.0).signum();
        let dz = (z - from.1).signum();
        let len = (x - from.0).abs() + (z - from.1).abs();
        for n in 0..=len {
            let (cx, cz) = (from.0 + n * dx, from.1 + n * dz);
            for side in -2..=2 {
                let (px, pz) = (cx + side * dz, cz - side * dx);
                let mat = if side == 0 && n % 5 == 0 {
                    GOLD
                } else if (-4..21).contains(&px) {
                    PLANK
                } else {
                    PATH
                };
                b.insert((px, 88, pz), mat);
            }
            if (-4..21).contains(&cx) && n % 5 == 0 {
                for side in [-3, 3] {
                    for y in 86..=90 {
                        b.insert((cx + side * dz, y, cz - side * dx), LOG);
                    }
                }
            }
        }
        let color = [LIME, BLUE, GOLD, RED, WHITE, GOLD][idx];
        for s in -3i32..=3 {
            for y in 89..=94 {
                if s.abs() == 3 || y == 94 {
                    b.insert(
                        (x + s * dz, y, z - s * dx),
                        if y == 94 && s == 0 { LIGHT } else { color },
                    );
                }
            }
        }
        from = (x, z);
    }
    // Two missing planks across the bridge create separate one-block jumps.
    for z in [-5, 6] {
        for x in 6..=10 {
            b.remove(&(x, 88, z));
        }
    }
    for x in -24..=-20 {
        b.insert((x, 89, -8), LOG);
    }
    for x in 16..=23 {
        for z in 3..=8 {
            b.insert((x, 98, z), PLANK);
        }
    }
    b
}

pub const GACHA_SHRINE: (i32, i32, i32) = (43, 78, 0);
pub const PARKOUR_START: (i32, i32, i32) = (40, 78, 0);
pub const PARKOUR_CHECKPOINTS: [(i32, i32, i32, &str); 8] = [
    (56, 79, 0, "Cổng Rừng Thông"),
    (66, 85, 26, "Đảo Rừng Nổi"),
    (40, 93, 58, "Cầu Băng Tuyết"),
    (-2, 101, 60, "Pháo Đài Gạch"),
    (-52, 109, 26, "Đảo Sắc Màu"),
    (-54, 125, -18, "Tháp Xoắn Ốc"),
    (-6, 129, -56, "Cầu Trời Mây"),
    (50, 136, -38, "Đỉnh Khải Hoàn"),
];

fn glyph_3x5(c: char) -> [u8; 5] {
    match c {
        'A' => [0b010, 0b101, 0b111, 0b101, 0b101],
        'C' => [0b011, 0b100, 0b100, 0b100, 0b011],
        'G' => [0b011, 0b100, 0b101, 0b101, 0b011],
        'H' => [0b101, 0b101, 0b111, 0b101, 0b101],
        'I' => [0b111, 0b010, 0b010, 0b010, 0b111],
        'K' => [0b101, 0b101, 0b110, 0b101, 0b101],
        'M' => [0b101, 0b111, 0b111, 0b101, 0b101],
        'O' => [0b010, 0b101, 0b101, 0b101, 0b010],
        'P' => [0b110, 0b101, 0b110, 0b100, 0b100],
        'R' => [0b110, 0b101, 0b110, 0b101, 0b101],
        'S' => [0b011, 0b100, 0b010, 0b001, 0b110],
        'T' => [0b111, 0b010, 0b010, 0b010, 0b010],
        'U' => [0b101, 0b101, 0b101, 0b101, 0b111],
        '2' => [0b110, 0b001, 0b010, 0b100, 0b111],
        '3' => [0b110, 0b001, 0b010, 0b001, 0b110],
        '6' => [0b011, 0b100, 0b111, 0b101, 0b111],
        '.' => [0b000, 0b000, 0b000, 0b000, 0b010],
        _ => [0; 5],
    }
}

/// Vẽ chữ nổi 3D phát sáng trên bầu trời (font 3x5 có lớp nền màu).
fn draw_text_3d(
    b: &mut Blocks,
    text: &str,
    origin: (i32, i32, i32),
    dx: i32,
    dz: i32,
    front: i32,
    back: i32,
) {
    let mut cursor = 0i32;
    for ch in text.chars() {
        let rows = glyph_3x5(ch);
        for (ry, &mask) in rows.iter().enumerate() {
            let y = origin.1 + (4 - ry as i32);
            for rx in 0..3 {
                if (mask >> (2 - rx)) & 1 != 0 {
                    let step = cursor + rx;
                    let x = origin.0 + step * dx;
                    let z = origin.2 + step * dz;
                    b.insert((x, y, z), front);
                    b.insert((x - dz, y, z + dx), back);
                }
            }
        }
        cursor += if ch == ' ' { 2 } else { 4 };
    }
}

fn pad(b: &mut Blocks, cx: i32, y: i32, cz: i32, r: i32, top: i32, under: i32) {
    for dx in -r..=r {
        for dz in -r..=r {
            b.insert((cx + dx, y, cz + dz), top);
            if under != 0 && dx.abs() + dz.abs() <= r {
                b.insert((cx + dx, y - 1, cz + dz), under);
            }
        }
    }
}

/// Tạo chuỗi bậc nhảy parkour 3x3 nối mép ngoài của hai đảo, đảm bảo khoảng hở mép 1..2 block và dy <= 1.
fn parkour_chain(
    b: &mut Blocks,
    from: (i32, i32, i32),
    from_r: i32,
    to: (i32, i32, i32),
    to_r: i32,
    mat_a: i32,
    mat_b: i32,
    under: i32,
) {
    let dx_all = (to.0 - from.0) as f64;
    let dz_all = (to.2 - from.2) as f64;
    let dist_all = dx_all.hypot(dz_all).max(1.0);
    let (ux, uz) = (dx_all / dist_all, dz_all / dist_all);

    let start_off = from_r as f64 + 3.2;
    let end_off = to_r as f64 + 3.2;
    let usable = (dist_all - start_off - end_off).max(0.0);

    let dy_total = (to.1 - from.1) as f64;
    let min_jumps = dy_total.abs().round() as i32;
    let target_jumps = ((usable / 4.2).round() as i32 + 2).max(min_jumps).max(2);
    let n_pads = target_jumps - 1;

    for i in 0..n_pads {
        let t = if n_pads == 1 {
            0.5
        } else {
            i as f64 / (n_pads - 1) as f64
        };
        let d = start_off + t * usable;
        let x = from.0 + (ux * d).round() as i32;
        let z = from.2 + (uz * d).round() as i32;
        let yt = (i + 1) as f64 / (n_pads + 1) as f64;
        let y = from.1 + (dy_total * yt).round() as i32;
        let m = if i % 2 == 0 { mat_a } else { mat_b };
        pad(b, x, y, z, 1, m, under);
    }
}

fn pine_tree(b: &mut Blocks, x: i32, base_y: i32, z: i32, h: i32) {
    for y in base_y..base_y + h {
        b.insert((x, y, z), LOG);
    }
    for (dy, r) in [(2, 2i32), (3, 2), (4, 1), (5, 1), (6, 0)] {
        for dx in -r..=r {
            for dz in -r..=r {
                if dx.abs() + dz.abs() <= r + 1 {
                    b.insert((x + dx, base_y + dy + (h - 6).max(0), z + dz), LEAVES);
                }
            }
        }
    }
}

/// Bản đồ khám phá mở rộng (121 chunks) + Parkour 8 chặng trên không + Đền Gacha + Chữ nổi 3D.
#[allow(dead_code)]
pub fn parkour_world() -> Blocks {
    let mut b = Blocks::new();

    // 1. Cầu gỗ nối từ khán đài Lobby (x=25) sang Đảo Khám Phá (x=35)
    for x in 25..=34 {
        for z in -2..=2 {
            b.insert((x, 77, z), if z == 0 && x % 2 == 0 { GOLD } else { PLANK });
        }
        if x % 3 == 0 {
            for z in [-2, 2] {
                b.insert((x, 78, z), LOG);
                b.insert((x, 79, z), LIGHT);
            }
        }
    }

    // 2. Đảo Trung Tâm Khám Phá (x=35..52, z=-9..9, y=77)
    for x in 35i32..=52 {
        for z in -9i32..=9 {
            let edge = x == 35 || x == 52 || z.abs() == 9;
            let mat = if edge {
                STONE
            } else if z.abs() <= 1 || (x - 43).abs() <= 1 {
                PATH
            } else {
                GRASS
            };
            b.insert((x, 77, z), mat);
            if (x - 43).abs() + z.abs() <= 12 {
                b.insert((x, 76, z), STONE);
            }
        }
    }
    for (tx, tz, h) in [(37, -7, 6), (37, 7, 7), (50, -7, 6), (50, 7, 7)] {
        pine_tree(&mut b, tx, 78, tz, h);
    }

    // 3. Đền Gacha tại (43, 78, 0)
    for dx in -2i32..=2 {
        for dz in -2i32..=2 {
            b.insert((43 + dx, 77, dz), GOLD);
            if dx.abs() == 2 && dz.abs() == 2 {
                for y in 78..=82 {
                    b.insert((43 + dx, y, dz), BRICKS);
                }
                b.insert((43 + dx, 83, dz), LIGHT);
            }
        }
    }
    b.insert((43, 78, 0), GOLD);
    b.insert((43, 79, 0), LIGHT);

    // 4. Chữ nổi 3D phát sáng trên bầu trời
    draw_text_3d(&mut b, "GACHA", (43, 86, -9), 0, 1, LIGHT, GOLD);
    draw_text_3d(&mut b, "RK 26.3", (36, 102, -13), 0, 1, LIGHT, RED);
    draw_text_3d(&mut b, "PARKOUR", (56, 87, -13), 0, 1, LIGHT, LIME);

    // 5. Đường chạy Parkour 8 chặng quanh quần đảo trên không
    let cps = PARKOUR_CHECKPOINTS;

    // Hub (mép x=52, y=77) -> Chặng 1 (mép x=54, y=78): 1 bước nhảy sạch cách 1 block
    // Chặng 1 -> Chặng 2: Rừng Thông Nổi (6 bước leo đều, không chồng bậc)
    parkour_chain(
        &mut b,
        (cps[0].0, cps[0].1 - 1, cps[0].2),
        2,
        (cps[1].0, cps[1].1 - 1, cps[1].2),
        4,
        GRASS,
        LOG,
        STONE,
    );
    pad(&mut b, cps[1].0, cps[1].1 - 1, cps[1].2, 4, GRASS, STONE);
    // Đặt 2 cây thông ở góc ngoài để tán lá không chắn hướng nhảy từ Chặng 1 tới
    pine_tree(&mut b, cps[1].0 + 4, cps[1].1, cps[1].2 + 3, 7);
    pine_tree(&mut b, cps[1].0 - 4, cps[1].1, cps[1].2 - 2, 6);

    // Chặng 2 -> Chặng 3: Cầu Băng Tuyết
    parkour_chain(
        &mut b,
        (cps[1].0, cps[1].1 - 1, cps[1].2),
        4,
        (cps[2].0, cps[2].1 - 1, cps[2].2),
        4,
        ICE,
        SNOW,
        STONE,
    );
    pad(&mut b, cps[2].0, cps[2].1 - 1, cps[2].2, 4, SNOW, ICE);
    for dy in 0..7 {
        b.insert((cps[2].0 + 4, cps[2].1 + dy, cps[2].2 + 4), if dy == 6 { LIGHT } else { ICE });
        b.insert((cps[2].0 - 4, cps[2].1 + dy, cps[2].2 + 4), if dy == 6 { LIGHT } else { ICE });
    }

    // Chặng 3 -> Chặng 4: Pháo Đài Gạch
    parkour_chain(
        &mut b,
        (cps[2].0, cps[2].1 - 1, cps[2].2),
        4,
        (cps[3].0, cps[3].1 - 1, cps[3].2),
        4,
        BRICKS,
        STONE,
        STONE,
    );
    pad(&mut b, cps[3].0, cps[3].1 - 1, cps[3].2, 4, BRICKS, STONE);
    for (ox, oz) in [(-4, -4), (4, -4), (-4, 4), (4, 4)] {
        for y in 0..5 {
            b.insert((cps[3].0 + ox, cps[3].1 + y, cps[3].2 + oz), BRICKS);
        }
        b.insert((cps[3].0 + ox, cps[3].1 + 5, cps[3].2 + oz), LIGHT);
    }

    // Chặng 4 -> Chặng 5: Đảo Sắc Màu
    parkour_chain(
        &mut b,
        (cps[3].0, cps[3].1 - 1, cps[3].2),
        4,
        (cps[4].0, cps[4].1 - 1, cps[4].2),
        4,
        RED,
        LIME,
        BLUE,
    );
    pad(&mut b, cps[4].0, cps[4].1 - 1, cps[4].2, 4, BLUE, WHITE);

    // Chặng 5 -> Chặng 6: Tháp Xoắn Ốc (bán kính 6.8, góc xoay 0.62 rad -> không chồng bậc)
    let (tcx, tcz) = (-54i32, -18i32);
    let start_ang = std::f64::consts::FRAC_PI_2;
    let first_x = tcx + (start_ang.cos() * 6.8).round() as i32;
    let first_z = tcz + (start_ang.sin() * 6.8).round() as i32;
    parkour_chain(
        &mut b,
        (cps[4].0, cps[4].1 - 1, cps[4].2),
        4,
        (first_x, 109, first_z),
        1,
        GOLD,
        WHITE,
        STONE,
    );
    for y in 107..=126 {
        for dx in -1i32..=1 {
            for dz in -1i32..=1 {
                let m = if dx == 0 && dz == 0 { LIGHT } else { BRICKS };
                b.insert((tcx + dx, y, tcz + dz), m);
            }
        }
    }
    for step in 0..16 {
        let ang = start_ang - (step as f64) * 0.62;
        let rx = tcx + (ang.cos() * 6.8).round() as i32;
        let rz = tcz + (ang.sin() * 6.8).round() as i32;
        let ry = 109 + step;
        pad(&mut b, rx, ry, rz, 1, if step % 2 == 0 { GOLD } else { PLANK }, STONE);
    }
    pad(&mut b, cps[5].0, cps[5].1 - 1, cps[5].2, 4, GOLD, BRICKS);

    // Chặng 6 -> Chặng 7: Cầu Trời Mây
    parkour_chain(
        &mut b,
        (cps[5].0, cps[5].1 - 1, cps[5].2),
        4,
        (cps[6].0, cps[6].1 - 1, cps[6].2),
        4,
        WHITE,
        PLANK,
        WHITE,
    );
    pad(&mut b, cps[6].0, cps[6].1 - 1, cps[6].2, 4, WHITE, SNOW);

    // Chặng 7 -> Chặng 8: Đèo Tuyết Cổ -> Đỉnh Khải Hoàn
    parkour_chain(
        &mut b,
        (cps[6].0, cps[6].1 - 1, cps[6].2),
        4,
        (cps[7].0, cps[7].1 - 1, cps[7].2),
        6,
        SNOW,
        GOLD,
        ICE,
    );

    // 6. Đền Đỉnh Khải Hoàn (13x13) tại Chặng 8 + Chữ nổi "SUMMIT"
    let (sx, sy, sz) = (cps[7].0, cps[7].1 - 1, cps[7].2);
    pad(&mut b, sx, sy, sz, 6, GOLD, BRICKS);
    pad(&mut b, sx, sy, sz, 3, WHITE, GOLD);
    b.insert((sx, sy, sz), LIGHT);
    for (ox, oz) in [(-5, -5), (5, -5), (-5, 5), (5, 5)] {
        for y in 1..=7 {
            b.insert((sx + ox, sy + y, sz + oz), if y == 7 { LIGHT } else { GOLD });
        }
    }
    draw_text_3d(&mut b, "SUMMIT", (sx - 11, sy + 9, sz - 6), 1, 0, LIGHT, GOLD);

    // 7. Đánh dấu từng Checkpoint bằng bệ sáng LIME + cột đèn
    for &(cx, cy, cz, _) in &cps {
        pad(&mut b, cx, cy - 1, cz, 2, LIME, GOLD);
        b.insert((cx, cy - 1, cz), LIGHT);
    }

    b
}
