use crate::{
    games::*,
    net::{Event, Guest, Shared, LOBBY},
    wire::*,
};
use std::{collections::BTreeMap, net::Shutdown, sync::Arc};
struct Player {
    id: i32,
    g: Guest,
    pos: Pos,
    serial: i32,
    ready: bool,
    moved: bool,
    punch: bool,
    wins: i32,
    rows: [String; 5],
    menu_open: bool,
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
}
pub struct Arena {
    players: BTreeMap<i32, Player>,
    max: usize,
    list: Vec<Game>,
    index: usize,
    round: Option<Round>,
    seed: u64,
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
    pub fn event(&mut self, event: Event) {
        match event {
            Event::Chat(id, msg) => {
                if msg == "!menu" {
                    if let Some(p) = self.players.get_mut(&id) {
                        p.menu_open = true;
                        p.send(crate::menu::open_packets());
                    }
                    return;
                }
                if let Some(p) = self.players.get(&id) {
                    self.cast(chat(&format!("<{}> {}", p.g.name, msg)));
                }
            }
            Event::MenuClose(id) => {
                if let Some(p) = self.players.get_mut(&id) {
                    p.menu_open = false;
                }
            }
            Event::MenuClick(id, slot) => {
                let Some(p) = self.players.get_mut(&id) else {
                    return;
                };
                if !p.menu_open {
                    return;
                }
                p.menu_open = false;
                p.push(crate::menu::close_packet());
                if let Some(&game) = GAMES.get(slot as usize) {
                    if let Some(pos) = self.list.iter().position(|&g| g == game) {
                        self.index = pos;
                        let name = game.meta().1;
                        let who = p.g.name.clone();
                        self.cast(chat(&format!("{who} đã chọn ván kế: {name}")));
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
                let mut p = Player {
                    id,
                    g,
                    pos: LOBBY,
                    serial: 0,
                    ready: false,
                    moved: false,
                    punch: false,
                    wins: 0,
                    rows: Default::default(),
                    menu_open: false,
                };
                let mut join = include_bytes!("../assets/join.bin").to_vec();
                join[..4].copy_from_slice(&id.to_be_bytes());
                p.push(
                    [
                        packet(0x32, &join),
                        packet(0x27, &[13, 0, 0, 0, 0]),
                        packet(0x61, &[6]),
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
                p.push(chat("Nhấn T để trò chuyện với mọi người."));
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
            [id] => format!("{} chiến thắng!", self.players[id].g.name),
            many => format!("{} người cùng thắng!", many.len()),
        };
        for id in winners {
            if let Some(p) = self.players.get_mut(&id) {
                p.wins += 1
            }
        }
        self.announce("KẾT THÚC", &msg, "gold");
        for p in self.players.values_mut() {
            p.warp(LOBBY)
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
        for p in self.players.values_mut() {
            p.poll();
            let active = self
                .round
                .as_ref()
                .is_some_and(|r| r.runners.get(&p.id).is_some_and(|v| v.alive));
            if !active && self.round.as_ref().is_some_and(|r| r.game == Game::Rally) {
                if p.ready
                    && (!(16.0..24.0).contains(&p.pos.x)
                        || !(3.0..9.0).contains(&p.pos.z)
                        || !(98.9..103.).contains(&p.pos.y))
                {
                    p.warp(crate::course::balcony());
                }
            } else if !active
                && p.ready
                && (p.pos.y < 77.
                    || p.pos.y > 85.
                    || p.pos.x < 17.
                    || p.pos.x > 25.
                    || p.pos.z.abs() > 4.)
            {
                p.warp(LOBBY)
            }
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
                .filter(|p| p.ready)
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
                    format!("Thắng phiên này: {}", p.wins),
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
