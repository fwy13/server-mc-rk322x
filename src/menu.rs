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

const ICONS: [i32; 15] = [
    160, 161, 162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172, 173, 174,
];
const ICON_BORDER: i32 = 160;
const ICON_POINTER: i32 = 164;
const ICON_PARKOUR: i32 = 165;
const ICON_GACHA: i32 = 170;
const ICON_LOBBY: i32 = 163;

pub struct GachaPrize {
    pub name: &'static str,
    pub color: &'static str,
    pub badge: &'static str,
    pub coins: i32,
    pub icon: i32,
    pub weight: u32,
    pub legendary: bool,
}

pub const PRIZES: [GachaPrize; 10] = [
    GachaPrize { name: "+5 Xu an ủi", color: "white", badge: "", coins: 5, icon: 160, weight: 22, legendary: false },
    GachaPrize { name: "+12 Xu may mắn", color: "green", badge: "", coins: 12, icon: 161, weight: 18, legendary: false },
    GachaPrize { name: "Danh hiệu [Tân Binh]", color: "white", badge: "[Tân Binh]", coins: 5, icon: 162, weight: 15, legendary: false },
    GachaPrize { name: "+20 Xu bạc", color: "aqua", badge: "", coins: 20, icon: 163, weight: 12, legendary: false },
    GachaPrize { name: "Danh hiệu [Thợ Săn]", color: "aqua", badge: "[Thợ Săn]", coins: 8, icon: 165, weight: 9, legendary: false },
    GachaPrize { name: "Danh hiệu [Phong Vân]", color: "aqua", badge: "[Phong Vân]", coins: 10, icon: 166, weight: 7, legendary: false },
    GachaPrize { name: "+35 Xu vàng", color: "light_purple", badge: "", coins: 35, icon: 168, weight: 6, legendary: false },
    GachaPrize { name: "Danh hiệu [Băng Vương]", color: "light_purple", badge: "[Băng Vương]", coins: 15, icon: 169, weight: 4, legendary: false },
    GachaPrize { name: "Danh hiệu [Hỏa Long]", color: "gold", badge: "[Hỏa Long]", coins: 20, icon: 171, weight: 3, legendary: false },
    GachaPrize { name: "★ HUYỀN THOẠI ★ (+100 Xu)", color: "gold", badge: "[★HUYỀN THOẠI★]", coins: 100, icon: 174, weight: 4, legendary: true },
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
        // Khi step dừng ở 17 (tổng 17 bước quay), ô giữa (slot 13) là reel[17 + 4] = reel[21].
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
                4 | 22 => Some(ICON_POINTER),
                9..=17 => {
                    let idx = self.reel[self.step + (i - 9)];
                    Some(PRIZES[idx].icon)
                }
                _ => Some(ICON_BORDER),
            };
            content.extend(slot(item));
        }
        content.extend(slot(None));
        packet(CB_SET_CONTAINER_CONTENT, &content)
    }
}

fn slot(item_id: Option<i32>) -> Buf {
    match item_id {
        None => vi(0),
        Some(id) => {
            let mut b = vi(1);
            b.extend(vi(id));
            b.extend(vi(0));
            b.extend(vi(0));
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
                    Some(ICONS[i])
                } else if i as i16 == SLOT_PARKOUR {
                    Some(ICON_PARKOUR)
                } else if i as i16 == SLOT_GACHA {
                    Some(ICON_GACHA)
                } else if i as i16 == SLOT_LOBBY {
                    Some(ICON_LOBBY)
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
    open.extend(component("✦ RƯƠNG GACHA MAY MẮN ✦", "gold"));
    packet(CB_OPEN_SCREEN, &open)
}

pub fn close_packet() -> Buf {
    packet(CB_CLOSE_CONTAINER, &[WINDOW_ID])
}
