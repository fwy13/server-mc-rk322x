use crate::{
    games::*,
    layout::{GACHA_SHRINE, PARKOUR_CHECKPOINTS, PARKOUR_START},
    menu::{
        close_packet, open_gacha_screen, open_packets, GachaSpin, MenuKind, GACHA_COST, PRIZES,
        SLOT_GACHA, SLOT_LOBBY, SLOT_PARKOUR,
    },
    net::{Event, Guest, Shared, LOBBY},
    wire::*,
};
use std::{collections::BTreeMap, net::Shutdown, sync::Arc};

const BOSS_UUID: [u8; 16] = *b"RKFESTIVALPARK01";

struct Player {
    id: i32,
    g: Guest,
    pos: Pos,
    serial: i32,
    ready: bool,
    moved: bool,
    punch: bool,
    wins: i32,
    coins: i32,
    badge: &'static str,
    exploring: bool,
    cp: usize,
    parkour_anchor: Pos,
    boss_shown: bool,
    gacha: Option<GachaSpin>,
    gacha_cd: u64,
    rows: [String; 5],
    menu: Option<MenuKind>,
}
impl Player {
    fn send(&self, data: Shared) {
        if self.g.out.try_send(data).is_err() {
            let _ = self.g.socket.shutdown(Shutdown::Both);
        }
    }
    fn push(&self, b: Buf) {
        self.send(Arc::from(b))
    }
    fn warp(&mut self, p: Pos) {
        self.serial = self.serial.wrapping_add(1);
        self.pos = p;
        self.moved = true;
        let mut s = self.g.input.lock().unwrap();
        s.pos = p;
        s.dirty = false;
        drop(s);
        self.push(teleport(self.serial, p))
    }
    fn poll(&mut self) {
        let mut s = self.g.input.lock().unwrap();
        self.punch = std::mem::take(&mut s.punch);
        self.ready = s.ready && s.ack == self.serial;
        if self.ready && s.dirty {
            self.pos = s.pos;
            self.moved = true;
            s.dirty = false
        }
    }
    fn display_name(&self) -> String {
        if self.badge.is_empty() {
            self.g.name.clone()
        } else {
            format!("{} {}", self.badge, self.g.name)
        }
    }
    fn sync_bossbar(&mut self) {
        if self.exploring {
            let total = PARKOUR_CHECKPOINTS.len();
            let pct = self.cp as f32 / total as f32;
            let label = format!(
                "✦ PARKOUR · Chặng {}/{} · {} Xu ✦",
                self.cp, total, self.coins
            );
            if !self.boss_shown {
                self.boss_shown = true;
                self.push(boss_bar_add(&BOSS_UUID, &label, "gold", pct, 4));
            } else {
                self.push(boss_bar_update(&BOSS_UUID, &label, "gold", pct));
            }
        } else if self.boss_shown {
            self.boss_shown = false;
            self.push(boss_bar_remove(&BOSS_UUID));
        }
    }
}
pub struct Arena {
    players: BTreeMap<i32, Player>,
    max: usize,
    list: Vec<Game>,
    index: usize,
    round: Option<Round>,
    seed: u64,
    now: u64,
    floor: [i32; N],
    lower: [i32; N],
    props: [[i32; N]; 2],
    start: Option<u64>,
    hud: u64,
    sync: u64,
    keep: u64,
    countdown: u64,
    results_until: u64,
}
impl Arena {
    pub fn new(max: usize, list: Vec<Game>, seed: u64) -> Self {
        Self {
            players: BTreeMap::new(),
            max,
            list,
            index: 0,
            round: None,
            seed,
            now: 0,
            floor: [WHITE; N],
            lower: [0; N],
            props: [[0; N]; 2],
            start: None,
            hud: 0,
            sync: 0,
            keep: 0,
            countdown: 0,
            results_until: 0,
        }
    }
    pub fn online(&self) -> usize {
        self.players.len()
    }
    fn cast(&self, b: Buf) {
        if !b.is_empty() {
            let data: Shared = Arc::from(b);
            for p in self.players.values() {
                p.send(data.clone())
            }
        }
    }
    fn announce(&self, main: &str, sub: &str, color: &str) {
        self.cast([title(main, sub, color), chat(&format!("{main} · {sub}"))].concat());
        println!("{main} · {sub}")
    }
    fn next_seed(&mut self) -> u64 {
        self.seed = self
            .seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.seed
    }
    fn open_gacha(&mut self, id: i32) {
        let now = self.now;
        let seed = self.next_seed() ^ (id as u64).wrapping_mul(0x9e3779b97f4a7c15);
        let Some(p) = self.players.get_mut(&id) else {
            return;
        };
        if p.gacha.is_some() {
            return;
        }
        if p.coins < GACHA_COST {
            p.push(action_bar(
                &format!(
                    "Cần {} Xu để quay Gacha (hiện có {} Xu)!",
                    GACHA_COST, p.coins
                ),
                "red",
            ));
            return;
        }
        p.coins -= GACHA_COST;
        p.gacha_cd = now + 1500;
        let spin = GachaSpin::new(seed, now);
        p.push([open_gacha_screen(), spin.frame()].concat());
        p.push(action_bar("✦ Đang quay rương Gacha... ✦", "gold"));
        p.menu = Some(MenuKind::Gacha);
        p.gacha = Some(spin);
        p.sync_bossbar();
    }
    fn enter_parkour(&mut self, id: i32) {
        if let Some(p) = self.players.get_mut(&id) {
            p.exploring = true;
            let start = Pos::at(
                PARKOUR_START.0 as f64 + 0.5,
                PARKOUR_START.1 as f64,
                PARKOUR_START.2 as f64 + 0.5,
            );
            if p.cp == 0 {
                p.parkour_anchor = start;
            }
            p.warp(p.parkour_anchor);
            p.sync_bossbar();
            p.push(title(
                "KHÁM PHÁ & PARKOUR",
                "Chinh phục 8 chặng · Gõ !lobby để về đấu trường",
                "green",
            ));
        }
    }
    fn leave_parkour(&mut self, id: i32) {
        if let Some(p) = self.players.get_mut(&id) {
            p.exploring = false;
            p.sync_bossbar();
            p.warp(LOBBY);
            p.push(action_bar("Đã quay về sảnh đấu trường!", "aqua"));
        }
    }
    pub fn event(&mut self, event: Event) {
        match event {
            Event::Chat(id, msg) => {
                match msg.as_str() {
                    "!menu" => {
                        if let Some(p) = self.players.get_mut(&id) {
                            p.gacha = None;
                            p.menu = Some(MenuKind::Games);
                            p.send(open_packets());
                        }
                        return;
                    }
                    "!gacha" => {
                        self.open_gacha(id);
                        return;
                    }
                    "!parkour" => {
                        self.enter_parkour(id);
                        return;
                    }
                    "!lobby" => {
                        self.leave_parkour(id);
                        return;
                    }
                    _ => {}
                }
                if let Some(p) = self.players.get(&id) {
                    self.cast(chat(&format!("<{}> {}", p.display_name(), msg)));
                }
            }
            Event::MenuClose(id) => {
                if let Some(p) = self.players.get_mut(&id) {
                    p.menu = None;
                    if let Some(spin) = p.gacha.take() {
                        if !spin.rewarded {
                            let prize = &PRIZES[spin.prize_idx];
                            p.coins += prize.coins;
                            if !prize.badge.is_empty() {
                                p.badge = prize.badge;
                            }
                            p.push(chat(&format!("✦ Gacha: Bạn nhận được {}!", prize.name)));
                            p.sync_bossbar();
                        }
                    }
                }
            }
            Event::MenuClick(id, slot) => {
                let kind = match self.players.get(&id).and_then(|p| p.menu) {
                    Some(k) => k,
                    None => return,
                };
                match kind {
                    MenuKind::Gacha => {
                        if let Some(p) = self.players.get_mut(&id) {
                            if let Some(spin) = &p.gacha {
                                if !spin.is_done() {
                                    // Chống desync item khi rương đang quay
                                    p.push(spin.frame());
                                    return;
                                }
                            }
                            p.menu = None;
                            p.gacha = None;
                            p.push(close_packet());
                        }
                    }
                    MenuKind::Games => {
                        if let Some(p) = self.players.get_mut(&id) {
                            p.menu = None;
                            p.push(close_packet());
                        }
                        if slot == SLOT_PARKOUR {
                            self.enter_parkour(id);
                            return;
                        }
                        if slot == SLOT_GACHA {
                            self.open_gacha(id);
                            return;
                        }
                        if slot == SLOT_LOBBY {
                            self.leave_parkour(id);
                            return;
                        }
                        if let Some(&game) = GAMES.get(slot as usize) {
                            if let Some(pos) = self.list.iter().position(|&g| g == game) {
                                self.index = pos;
                                let name = game.meta().1;
                                if let Some(p) = self.players.get(&id) {
                                    let who = p.display_name();
                                    self.cast(chat(&format!("{who} đã chọn ván kế: {name}")));
                                }
                            }
                        }
                    }
                }
            }
            Event::Leave(id) => {
                if let Some(p) = self.players.remove(&id) {
                    if let Some(r) = &mut self.round {
                        r.runners.remove(&id);
                    }
                    let mut rm = vec![1];
                    rm.extend(vi(id));
                    let mut tab = vec![1];
                    tab.extend(p.g.uuid);
                    self.cast([packet(0x4e, &rm), packet(0x46, &tab)].concat())
                }
            }
            Event::Join(id, g) => {
                if self.players.len() >= self.max
                    || self
                        .players
                        .values()
                        .any(|p| p.g.name.eq_ignore_ascii_case(&g.name))
                {
                    let _ = g.out.try_send(Arc::from(packet(
                        0x20,
                        &component("Sân đầy hoặc tên trùng", "red"),
                    )));
                    return;
                }
                let start_anchor = Pos::at(
                    PARKOUR_START.0 as f64 + 0.5,
                    PARKOUR_START.1 as f64,
                    PARKOUR_START.2 as f64 + 0.5,
                );
                let mut p = Player {
                    id,
                    g,
                    pos: LOBBY,
                    serial: 0,
                    ready: false,
                    moved: false,
                    punch: false,
                    wins: 0,
                    coins: 30,
                    badge: "",
                    exploring: false,
                    cp: 0,
                    parkour_anchor: start_anchor,
                    boss_shown: false,
                    gacha: None,
                    gacha_cd: 0,
                    rows: Default::default(),
                    menu: None,
                };
                let mut join = include_bytes!("../assets/join.bin").to_vec();
                join[..4].copy_from_slice(&id.to_be_bytes());
                p.push(
                    [
                        packet(0x32, &join),
                        packet(0x27, &[13, 0, 0, 0, 0]),
                        packet(0x61, &[8]),
                        packet(0x60, &[0, 0]),
                    ]
                    .concat(),
                );
                let rally = self.round.as_ref().is_some_and(|r| r.game == Game::Rally);
                p.send(crate::world::frames(rally));
                let mut all = vec![];
                for k in 0..N {
                    let x = (k % 18) as i32 - 9;
                    let z = (k / 18) as i32 - 9;
                    all.extend(block(x, 72, z, self.floor[k]));
                    all.extend(block(x, 66, z, self.lower[k]));
                    for layer in 0..2 {
                        if self.props[layer][k] != 0 {
                            all.extend(block(x, 73 + layer as i32, z, self.props[layer][k]))
                        }
                    }
                }
                all.extend(info(&p.g.uuid, &p.g.name));
                for q in self.players.values() {
                    all.extend(info(&q.g.uuid, &q.g.name));
                    all.extend(spawn(q.id, &q.g.uuid, q.pos))
                }
                self.cast([info(&p.g.uuid, &p.g.name), spawn(id, &p.g.uuid, p.pos)].concat());
                all.extend(board());
                all.extend(title(
                    "RK FESTIVAL",
                    if self.round.is_some() {
                        "Bạn sẽ chơi ở ván kế"
                    } else {
                        "Sẵn sàng vào sân!"
                    },
                    "gold",
                ));
                p.push(all);
                p.push(chat(
                    "Lệnh: !menu (chọn trò) · !parkour (khám phá) · !gacha (quay rương 10 Xu) · !lobby",
                ));
                if self.round.as_ref().is_some_and(|r| r.game == Game::Rally) {
                    p.warp(crate::course::balcony());
                } else {
                    p.warp(LOBBY);
                }
                self.players.insert(id, p);
            }
        }
    }
    fn terrain(&mut self) {
        let mut b = vec![];
        let idle = [WHITE; N];
        let empty = [0; N];
        let (f, l) = self
            .round
            .as_ref()
            .map_or((&idle, &empty), |r| (&r.floor, &r.lower));
        for k in 0..N {
            let (x, z) = ((k % 18) as i32 - 9, (k / 18) as i32 - 9);
            for j in 0..2 {
                let new = self.round.as_ref().map_or(0, |r| r.props[j][k]);
                if new != self.props[j][k] {
                    self.props[j][k] = new;
                    b.extend(block(x, 73 + j as i32, z, new))
                }
            }
            for (y, old, new) in [
                (72, &mut self.floor[k], f[k]),
                (66, &mut self.lower[k], l[k]),
            ] {
                if *old != new {
                    *old = new;
                    b.extend(block(x, y, z, new))
                }
            }
        }
        self.cast(b)
    }
    fn finish(&mut self, winners: Vec<i32>, now: u64) {
        self.results_until = now + 3000;
        let msg = match winners.as_slice() {
            [] => "Hết giờ · chưa có người thắng".to_owned(),
            [id] => format!("{} chiến thắng! (+10 Xu)", self.players[id].display_name()),
            many => format!("{} người cùng thắng! (+10 Xu)", many.len()),
        };
        for id in winners {
            if let Some(p) = self.players.get_mut(&id) {
                p.wins += 1;
                p.coins += 10;
            }
        }
        self.announce("KẾT THÚC", &msg, "gold");
        for p in self.players.values_mut() {
            if !p.exploring {
                p.warp(LOBBY)
            }
        }
        if self.round.as_ref().is_some_and(|r| r.game == Game::Rally) {
            let clear = crate::world::frames(false);
            for p in self.players.values() {
                p.send(clear.clone());
            }
        }
        self.round = None;
        self.start = None;
        self.countdown = 0;
        self.index = (self.index + 1) % self.list.len();
        self.terrain()
    }
    pub fn tick(&mut self, now: u64) {
        self.now = now;
        let is_rally = self.round.as_ref().is_some_and(|r| r.game == Game::Rally);
        let mut gacha_triggers = vec![];
        let mut broadcasts = vec![];

        for p in self.players.values_mut() {
            p.poll();

            // Cập nhật hoạt ảnh quay rương Gacha
                        let mut gacha_frame = None;
                        let mut gacha_prize = None;
                        let mut gacha_close = false;
                        if let Some(spin) = &mut p.gacha {
                            if spin.advance(now) {
                                gacha_frame = Some(spin.frame());
                                if spin.is_done() && !spin.rewarded {
                                    spin.rewarded = true;
                                    gacha_prize = Some(spin.prize_idx);
                                }
                            }
                            gacha_close = spin.close_at.is_some_and(|t| now >= t);
                        }
                        if let Some(frame) = gacha_frame {
                            p.push(frame);
                        }
                        if let Some(idx) = gacha_prize {
                            let prize = &PRIZES[idx];
                            p.coins += prize.coins;
                            if !prize.badge.is_empty() {
                                p.badge = prize.badge;
                            }
                            p.push(title("TRÚNG THƯỞNG!", prize.name, prize.color));
                            p.push(action_bar(
                                &format!("✦ Gacha: {} · Tổng: {} Xu ✦", prize.name, p.coins),
                                prize.color,
                            ));
                            p.push(chat(&format!(
                                "✦ Rương Gacha: Bạn nhận được {}!",
                                prize.name
                            )));
                            if prize.legendary {
                                broadcasts.push(format!(
                                    "★ {} vừa nổ hũ {} từ Rương Gacha! ★",
                                    p.display_name(),
                                    prize.name
                                ));
                            }
                        }
                        if gacha_close {
                            p.gacha = None;
                            if p.menu == Some(MenuKind::Gacha) {
                                p.menu = None;
                                p.push(close_packet());
                            }
                            p.sync_bossbar();
                        }
            }

            let active = self
                .round
                .as_ref()
                .is_some_and(|r| r.runners.get(&p.id).is_some_and(|v| v.alive));

            if !active && is_rally {
                if p.ready
                    && (!(16.0..24.0).contains(&p.pos.x)
                        || !(3.0..9.0).contains(&p.pos.z)
                        || !(98.9..103.).contains(&p.pos.y))
                {
                    p.warp(crate::course::balcony());
                }
            } else if !active && p.ready {
                // Tự động vào chế độ Khám phá / Parkour khi bước qua cầu gỗ (x > 25)
                if !p.exploring && p.pos.x > 25.0 && p.pos.y >= 76.0 {
                    p.exploring = true;
                    p.sync_bossbar();
                    p.push(action_bar(
                        "✦ Chế độ Khám phá · Đi tới Đền Gacha hoặc Parkour 8 chặng! ✦",
                        "green",
                    ));
                }

                if p.exploring {
                    // Rơi khỏi đảo -> quay về Checkpoint gần nhất
                    if p.pos.y < 58.0 || p.pos.x.abs() > 84.0 || p.pos.z.abs() > 84.0 {
                        p.warp(p.parkour_anchor);
                        p.push(action_bar("Đã quay về điểm lưu Parkour!", "yellow"));
                    } else {
                        // Tương tác Đền Gacha (43, 78, 0)
                        let gdx = p.pos.x - (GACHA_SHRINE.0 as f64 + 0.5);
                        let gdz = p.pos.z - (GACHA_SHRINE.2 as f64 + 0.5);
                        let gdy = (p.pos.y - GACHA_SHRINE.1 as f64).abs();
                        if gdx.hypot(gdz) <= 3.8 && gdy <= 2.5 {
                            if p.punch && now >= p.gacha_cd && p.gacha.is_none() {
                                gacha_triggers.push(p.id);
                            } else if p.gacha.is_none() && now % 500 < 50 {
                                p.push(action_bar(
                                    &format!(
                                        "✦ Đền Gacha · Nhấp chuột hoặc gõ !gacha ({} Xu/lượt · Bạn có {} Xu) ✦",
                                        GACHA_COST, p.coins
                                    ),
                                    "gold",
                                ));
                            }
                        }

                        // Kiểm tra chạm Checkpoint Parkour kế tiếp
                        if p.cp < PARKOUR_CHECKPOINTS.len() {
                            let (cx, cy, cz, cp_name) = PARKOUR_CHECKPOINTS[p.cp];
                            let d = (p.pos.x - cx as f64 - 0.5).hypot(p.pos.z - cz as f64 - 0.5);
                            if d <= 3.0 && (p.pos.y - cy as f64).abs() <= 2.5 {
                                p.cp += 1;
                                p.parkour_anchor =
                                    Pos::at(cx as f64 + 0.5, cy as f64, cz as f64 + 0.5);
                                if p.cp == PARKOUR_CHECKPOINTS.len() {
                                    p.coins += 30;
                                    p.wins += 1;
                                    p.cp = 0;
                                    p.parkour_anchor = Pos::at(
                                        PARKOUR_START.0 as f64 + 0.5,
                                        PARKOUR_START.1 as f64,
                                        PARKOUR_START.2 as f64 + 0.5,
                                    );
                                    p.push(title(
                                        "CHINH PHỤC ĐỈNH KHẢI HOÀN!",
                                        "+30 Xu · Bạn đã hoàn thành 8 chặng Parkour!",
                                        "gold",
                                    ));
                                    broadcasts.push(format!(
                                        "✦ {} đã chinh phục Đỉnh Khải Hoàn Parkour (+30 Xu)!",
                                        p.display_name()
                                    ));
                                } else {
                                    p.coins += 5;
                                    p.push(title(
                                        &format!("CHẶNG {}/8", p.cp),
                                        &format!("{cp_name} · +5 Xu"),
                                        "green",
                                    ));
                                }
                                p.sync_bossbar();
                            }
                        }
                    }
                } else if p.pos.y < 77.
                    || p.pos.y > 85.
                    || p.pos.x < 17.
                    || p.pos.z.abs() > 4.
                {
                    p.warp(LOBBY)
                }
            }
        }

        for id in gacha_triggers {
            self.open_gacha(id);
        }
        for msg in broadcasts {
            self.cast(chat(&msg));
        }

        if now >= self.keep {
            self.keep = now + 12000;
            self.cast(packet(0x2d, &now.to_be_bytes()))
        }
        if let Some(r) = self.round.as_mut() {
            for (&id, v) in &mut r.runners {
                if let Some(p) = self.players.get(&id) {
                    v.pos = p.pos;
                    v.punch = p.punch
                }
            }
            let actions = r.update(now);
            self.terrain();
            for action in actions {
                match action {
                    Action::Cue(main, sub) => self.announce(&main, &sub, "yellow"),
                    Action::Warp(id, p) => {
                        if let Some(v) = self.players.get_mut(&id) {
                            v.warp(p)
                        }
                    }
                    Action::Out(id) => {
                        if let Some(p) = self.players.get_mut(&id) {
                            p.warp(LOBBY);
                            p.push(title("ĐÃ BỊ LOẠI", "Theo dõi ván từ khán đài", "red"))
                        }
                    }
                    Action::End(w) => {
                        self.finish(w, now);
                        break;
                    }
                }
            }
        } else if now >= self.results_until {
            let ids: Vec<_> = self
                .players
                .values()
                .filter(|p| p.ready && !p.exploring)
                .map(|p| p.id)
                .collect();
            if ids.is_empty() {
                self.start = None;
                self.countdown = 0
            } else {
                let deadline = *self.start.get_or_insert(now + 8000);
                let game = self.list[self.index];
                let (_, name, rules) = game.meta();
                if now >= deadline {
                    self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                    let r = Round::new(game, &ids, now, self.seed);
                    self.announce(name, rules, "gold");
                    if game == Game::Rally {
                        let map = crate::world::frames(true);
                        for p in self.players.values_mut() {
                            p.exploring = false;
                            p.sync_bossbar();
                            p.send(map.clone());
                            if !r.runners.contains_key(&p.id) {
                                p.warp(crate::course::balcony());
                            }
                        }
                    }
                    for (&id, v) in &r.runners {
                        let p = self.players.get_mut(&id).unwrap();
                        p.warp(v.spawn);
                        if game == Game::Paint {
                            p.push(chat(if v.team == 0 {
                                "Bạn ở đội ĐỎ"
                            } else {
                                "Bạn ở đội XANH DƯƠNG"
                            }))
                        }
                    }
                    self.round = Some(r);
                    self.terrain()
                } else {
                    let left = (deadline - now).div_ceil(1000);
                    if self.countdown != left {
                        self.countdown = left;
                        if left <= 3 {
                            self.cast(title(&left.to_string(), name, "aqua"))
                        } else if left == 8 {
                            self.announce(name, rules, "aqua")
                        }
                    }
                }
            }
        }
        if now >= self.sync {
            self.sync = now + 100;
            let mut moves = vec![];
            let mut spans = BTreeMap::new();
            for (&id, p) in &mut self.players {
                if p.moved {
                    let a = moves.len();
                    moves.extend(movement(id, p.pos));
                    spans.insert(id, (a, moves.len()));
                    p.moved = false
                }
            }
            if !moves.is_empty() {
                for (&id, p) in &self.players {
                    if let Some(&(a, b)) = spans.get(&id) {
                        let data = [&moves[..a], &moves[b..]].concat();
                        if !data.is_empty() {
                            p.push(data)
                        }
                    } else {
                        p.push(moves.clone())
                    }
                }
            }
        }
        if now >= self.hud {
            self.hud = now + 250;
            let game = self.list[self.index];
            let live = self
                .round
                .as_ref()
                .map_or(0, |r| r.runners.values().filter(|v| v.alive).count());
            let online = self.players.len();
            for p in self.players.values_mut() {
                let (hint, value) = if let Some(r) = &self.round {
                    let personal = match r.runners.get(&p.id) {
                        None if p.exploring => {
                            format!("Parkour {}/8", p.cp)
                        }
                        None => "Khán giả".into(),
                        Some(v) if !v.alive => "Đã bị loại".into(),
                        Some(v) if game == Game::Rally => {
                            let (x, z, name) = crate::course::GATES[v.score as usize];
                            format!("{}/6 → {} ({},{})", v.score, name, x, z)
                        }
                        Some(v) if game == Game::Wave => format!("Tim: {}", v.hearts),
                        Some(v) if game == Game::Paint => {
                            format!("Đội {}", if v.team == 0 { "Đỏ" } else { "Xanh" })
                        }
                        Some(v) => format!("Điểm: {}", v.score),
                    };
                    (
                        if r.hint.is_empty() {
                            format!("Còn {}s", (r.end.saturating_sub(now)).div_ceil(1000))
                        } else {
                            format!("{} · {}s", r.hint, r.end.saturating_sub(now).div_ceil(1000))
                        },
                        personal,
                    )
                } else if p.exploring {
                    let next = PARKOUR_CHECKPOINTS[p.cp.min(PARKOUR_CHECKPOINTS.len() - 1)];
                    (
                        format!("→ {} ({},{})", next.3, next.0, next.2),
                        format!("Parkour {}/8", p.cp),
                    )
                } else {
                    (
                        format!(
                            "Chuẩn bị · {}s",
                            self.start.unwrap_or(now).saturating_sub(now).div_ceil(1000)
                        ),
                        "Khán đài".into(),
                    )
                };
                let rows = [
                    game.meta().1.into(),
                    format!("Online {online} · Còn {live}"),
                    hint,
                    value,
                    format!("Thắng: {} · Xu: {}", p.wins, p.coins),
                ];
                let mut msg = vec![];
                for (i, s) in rows.iter().enumerate() {
                    if p.rows[i] != *s {
                        msg.extend(row(i, s))
                    }
                }
                p.rows = rows;
                if !msg.is_empty() {
                    p.push(msg)
                }
            }
        }
    }
}
