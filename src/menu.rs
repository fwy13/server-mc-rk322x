use crate::{
    games::GAMES,
    net::Shared,
    wire::{component, packet, vi, Buf},
};
use std::sync::OnceLock;

// Lưu ý: Đối chiếu các Packet ID này với vanilla 26.3 (protocol 777) data reports.
pub const CB_CLOSE_CONTAINER: i32 = 0x12;
pub const CB_SET_CONTAINER_CONTENT: i32 = 0x13;
pub const CB_OPEN_SCREEN: i32 = 0x34;
pub const WINDOW_ID: u8 = 1;
const GENERIC_9X3: i32 = 2; // MenuType: generic_9x3

// Item protocol ID đại diện cho 15 trò trong GAMES (có thể tùy chỉnh theo registry 26.3).
const ICONS: [i32; 15] = [
    160, 161, 162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172, 173, 174,
];

fn slot(item_id: Option<i32>) -> Buf {
    match item_id {
        None => vi(0),
        Some(id) => {
            let mut b = vi(1); // item count = 1
            b.extend(vi(id)); // item protocol ID
            b.extend(vi(0)); // added components count
            b.extend(vi(0)); // removed components count
            b
        }
    }
}

pub fn open_packets() -> Shared {
    static CACHE: OnceLock<Shared> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            // 1. Open Screen
            let mut open = vi(WINDOW_ID as i32);
            open.extend(vi(GENERIC_9X3));
            open.extend(component("✦ CHỌN MINIGAME ✦", "gold"));

            // 2. Set Container Content (27 slots)
            let mut content = vi(WINDOW_ID as i32);
            content.extend(vi(1)); // state_id
            content.extend(vi(27)); // slot count
            for i in 0..27 {
                let item = if i < GAMES.len() {
                    Some(ICONS[i])
                } else {
                    None
                };
                content.extend(slot(item));
            }
            content.extend(slot(None)); // carried_item = empty

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

pub fn close_packet() -> Buf {
    packet(CB_CLOSE_CONTAINER, &[WINDOW_ID])
}
