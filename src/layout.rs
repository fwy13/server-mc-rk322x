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
