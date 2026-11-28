use crate::{
    games::{Rng, GAMES},
    net::Shared,
    wire::{component, packet, vi, Buf},
};
use std::sync::OnceLock;

// Packet ID chuẩn từ Minecraft Java 26.3 (protocol 777) packets.json.
pub const CB_CLOSE_CONTAINER: i32 = 0x11;
pub const CB_SET_CONTAINER_CONTENT: i32 = 0x12;
pub const CB_OPEN_SCREEN: i32 = 0x3c;
pub const WINDOW_ID: u8 = 1;
const GENERIC_9X3: i32 = 2; // MenuType: generic_9x3

pub const GACHA_COST: i32 = 10;
pub const SLOT_PARKOUR: i16 = 20;
pub const SLOT_GACHA: i16 = 22;
pub const SLOT_LOBBY: i16 = 24;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum MenuKind {
    Games,
    Gacha,
}

// Item protocol ID chuẩn từ Minecraft Java 26.3 registries.json.
const COMP_CUSTOM_NAME: i32 = 6;
const ITEM_GLASS_PANE: i32 = 628;
const ITEM_HOPPER: i32 = 834;
const ITEM_CHEST: i32 = 404;
const ITEM_COMPASS: i32 = 1151;
const ITEM_IRON_INGOT: i32 = 1018;
const ITEM_GOLD_INGOT: i32 = 1022;
const ITEM_EMERALD: i32 = 1013;
const ITEM_DIAMOND: i32 = 1012;
const ITEM_NETHERITE_INGOT: i32 = 1023;
const ITEM_ENCHANTED_APPLE: i32 = 1101;
const ITEM_NETHER_STAR: i32 = 1391;
const ITEM_TOTEM: i32 = 1454;

const ICONS: [i32; 15] = [
    ITEM_GOLD_INGOT,
    ITEM_EMERALD,
    ITEM_DIAMOND,
    ITEM_IRON_INGOT,
    ITEM_COMPASS,
    ITEM_NETHER_STAR,
    ITEM_ENCHANTED_APPLE,
    ITEM_CHEST,
    ITEM_GOLD_INGOT,
    ITEM_DIAMOND,
    ITEM_EMERALD,
    ITEM_IRON_INGOT,
    ITEM_TOTEM,
    ITEM_NETHERITE_INGOT,
    ITEM_COMPASS,
];

pub struct GachaPrize {
    pub name: &'static str,
    pub color: &'static str,
    pub badge: &'static str,
    pub coins: i32,
    pub icon: i32,
    pub count: i32,
    pub weight: u32,
    pub legendary: bool,
}

pub const PRIZES: [GachaPrize; 10] = [
    GachaPrize { name: "Phôi Sắt x3 (+5 Xu)", color: "white", badge: "", coins: 5, icon: ITEM_IRON_INGOT, count: 3, weight: 22, legendary: false },
    GachaPrize { name: "Thỏi Vàng x2 (+12 Xu)", color: "yellow", badge: "", coins: 12, icon: ITEM_GOLD_INGOT, count: 2, weight: 18, legendary: false },
    GachaPrize { name: "Danh hiệu [Tân Binh] + Thỏi Vàng", color: "white", badge: "[Tân Binh]", coins: 5, icon: ITEM_GOLD_INGOT, count: 1, weight: 15, legendary: false },
    GachaPrize { name: "Ngọc Lục Bảo x2 (+20 Xu)", color: "green", badge: "", coins: 20, icon: ITEM_EMERALD, count: 2, weight: 12, legendary: false },
    GachaPrize { name: "Danh hiệu [Thợ Săn] + Ngọc Lục Bảo", color: "aqua", badge: "[Thợ Săn]", coins: 8, icon: ITEM_EMERALD, count: 1, weight: 9, legendary: false },
    GachaPrize { name: "Kim Cương x1 (+25 Xu)", color: "aqua", badge: "", coins: 25, icon: ITEM_DIAMOND, count: 1, weight: 8, legendary: false },
    GachaPrize { name: "Táo Vàng Phù Phép (+35 Xu)", color: "light_purple", badge: "[Băng Vương]", coins: 35, icon: ITEM_ENCHANTED_APPLE, count: 1, weight: 6, legendary: false },
    GachaPrize { name: "Totem Bất Tử + [Phong Vân]", color: "gold", badge: "[Phong Vân]", coins: 25, icon: ITEM_TOTEM, count: 1, weight: 4, legendary: false },
    GachaPrize { name: "Thỏi Netherite + [Hỏa Long]", color: "gold", badge: "[Hỏa Long]", coins: 40, icon: ITEM_NETHERITE_INGOT, count: 1, weight: 3, legendary: false },
    GachaPrize { name: "★ HUYỀN THOẠI ★ Sao Nether (+100 Xu)", color: "gold", badge: "[★HUYỀN THOẠI★]", coins: 100, icon: ITEM_NETHER_STAR, count: 1, weight: 3, legendary: true },
];

const SPIN_DELAYS_MS: [u64; 17] = [
    65, 65, 70, 75, 80, 90, 100, 115, 135, 160, 190, 225, 265, 315, 375, 450, 540,
];

fn roll_prize(rng: &mut Rng) -> usize {
    let total: u32 = PRIZES.iter().map(|p| p.weight).sum();
    let mut pick = rng.pick(total as usize) as u32;
    for (i, p) in PRIZES.iter().enumerate() {
        if pick < p.weight {
            return i;
        }
        pick -= p.weight;
    }
    0
}

pub struct GachaSpin {
    reel: [usize; 26],
    pub step: usize,
    pub next_at: u64,
    state_id: i32,
    pub prize_idx: usize,
    pub rewarded: bool,
    pub close_at: Option<u64>,
}

impl GachaSpin {
    pub fn new(seed: u64, now: u64) -> Self {
        let mut rng = Rng::new(seed);
        let mut reel = [0usize; 26];
        for slot in &mut reel {
            *slot = roll_prize(&mut rng);
        }
        // Khi step dừng ở 17, ô giữa (slot 13) là reel[17 + 4] = reel[21].
        let prize_idx = reel[21];
        Self {
            reel,
            step: 0,
            next_at: now + SPIN_DELAYS_MS[0],
            state_id: 2,
            prize_idx,
            rewarded: false,
            close_at: None,
        }
    }

    pub fn advance(&mut self, now: u64) -> bool {
        if self.step >= SPIN_DELAYS_MS.len() || now < self.next_at {
            return false;
        }
        self.step += 1;
        self.state_id = self.state_id.wrapping_add(1);
        if self.step < SPIN_DELAYS_MS.len() {
            self.next_at = now + SPIN_DELAYS_MS[self.step];
        } else {
            self.close_at = Some(now + 2200);
        }
        true
    }

    pub fn is_done(&self) -> bool {
        self.step >= SPIN_DELAYS_MS.len()
    }

    pub fn frame(&self) -> Buf {
        let mut content = vi(WINDOW_ID as i32);
        content.extend(vi(self.state_id));
        content.extend(vi(27));
        for i in 0..27 {
            let item = match i {
                4 | 22 => Some((ITEM_HOPPER, 1, "▼ Ô TRÚNG THƯỞNG ▼", "gold")),
                9..=17 => {
                    let p = &PRIZES[self.reel[self.step + (i - 9)]];
                    Some((p.icon, p.count, p.name, p.color))
                }
                _ => Some((ITEM_GLASS_PANE, 1, " ", "gray")),
            };
            content.extend(slot(item));
        }
        content.extend(slot(None));
        packet(CB_SET_CONTAINER_CONTENT, &content)
    }
}

fn slot(item: Option<(i32, i32, &str, &str)>) -> Buf {
    match item {
        None => vi(0),
        Some((id, count, name, color)) => {
            let mut b = vi(count);
            b.extend(vi(id));
            if name.is_empty() {
                b.extend(vi(0));
                b.extend(vi(0));
            } else {
                b.extend(vi(1)); // components_to_add = 1
                b.extend(vi(0)); // components_to_remove = 0
                b.extend(vi(COMP_CUSTOM_NAME)); // minecraft:custom_name = 6
                b.extend(component(name, color));
            }
            b
        }
    }
}

pub fn open_packets() -> Shared {
    static CACHE: OnceLock<Shared> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            let mut open = vi(WINDOW_ID as i32);
            open.extend(vi(GENERIC_9X3));
            open.extend(component("✦ MENU RK FESTIVAL ✦", "gold"));

            let mut content = vi(WINDOW_ID as i32);
            content.extend(vi(1));
            content.extend(vi(27));
            for i in 0..27 {
                let item = if i < GAMES.len() {
                    Some((ICONS[i], 1, GAMES[i].meta().1, "aqua"))
                } else if i as i16 == SLOT_PARKOUR {
                    Some((ITEM_COMPASS, 1, "✦ Khám phá & Parkour 8 chặng ✦", "green"))
                } else if i as i16 == SLOT_GACHA {
                    Some((ITEM_CHEST, 1, "★ Quay Rương Gacha (10 Xu) ★", "gold"))
                } else if i as i16 == SLOT_LOBBY {
                    Some((ITEM_NETHER_STAR, 1, "Về Sảnh Chính (Hub)", "yellow"))
                } else {
                    None
                };
                content.extend(slot(item));
            }
            content.extend(slot(None));

            Shared::from(
                [
                    packet(CB_OPEN_SCREEN, &open),
                    packet(CB_SET_CONTAINER_CONTENT, &content),
                ]
                .concat(),
            )
        })
        .clone()
}

pub fn open_gacha_screen() -> Buf {
    let mut open = vi(WINDOW_ID as i32);
    open.extend(vi(GENERIC_9X3));
    open.extend(component("✦ RƯƠNG GACHA KIM CƯƠNG ✦", "gold"));
    packet(CB_OPEN_SCREEN, &open)
}

pub fn close_packet() -> Buf {
    packet(CB_CLOSE_CONTAINER, &[WINDOW_ID])
}
