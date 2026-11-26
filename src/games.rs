use crate::wire::Pos;
use std::collections::BTreeMap;
pub const SIDE: usize = 18;
pub const N: usize = SIDE * SIDE;
pub const TNT: i32 = 3834;
pub use crate::colors::*;
const DYES: [i32; 6] = [RED, GOLD, LIME, BLUE, WHITE, 17021];
const NAMES: [&str; 6] = ["ĐỎ", "VÀNG", "XANH LÁ", "XANH DƯƠNG", "TRẮNG", "TÍM"];
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Game {
    Color,
    Paint,
    Wave,
    Tnt,
    Memory,
    Beacon,
    Traffic,
    Treasure,
    King,
    Ice,
    Islands,
    Mine,
    Snow,
    Pipe,
    Rally,
}
use Game::*;
pub const GAMES: [Game; 15] = [
    Color, Paint, Wave, Tnt, Memory, Beacon, Traffic, Treasure, King, Ice, Islands, Mine, Snow,
    Pipe, Rally,
];
impl Game {
    pub fn meta(self) -> (&'static str, &'static str, &'static str) {
        match self {
            Rally => (
                "rally",
                "Đua sáu cổng",
                "Theo vạch vàng, vượt sáu cổng theo thứ tự",
            ),
            Color => ("color", "Sàn sắc màu", "Đứng trên màu được gọi khi hết giờ"),
            Paint => (
                "paint",
                "Đại chiến sơn sàn",
                "Chạy để sơn ô cho đội Đỏ hoặc Xanh",
            ),
            Wave => (
                "wave",
                "Né sóng",
                "Tránh vạch đỏ đang quét; tìm khe để đi qua",
            ),
            Tnt => ("tnt", "TNT Run", "Chạy liên tục trên hai tầng TNT"),
            Memory => (
                "memory",
                "Sàn trí nhớ",
                "Nhớ vị trí 6 màu trước khi sàn trắng",
            ),
            Beacon => (
                "beacon",
                "Săn đốm sáng",
                "Chạm ô vàng trước người khác để lấy điểm",
            ),
            Traffic => (
                "traffic",
                "Đèn xanh đèn đỏ",
                "Chạy từ nam lên bắc; dừng khi đèn đỏ",
            ),
            Treasure => (
                "treasure",
                "Kho báu tập thể",
                "Cùng dò ô chưa mở để tìm đủ 5 chìa khóa",
            ),
            King => (
                "king",
                "Vua giữ đài",
                "Đứng trong vùng xanh để tích điểm mỗi giây",
            ),
            Ice => ("ice", "Băng tan", "Né các ô báo đỏ trước khi băng vỡ"),
            Islands => ("islands", "Đảo thu nhỏ", "Tới đảo xanh trước khi nước dâng"),
            Snow => (
                "snow",
                "Tìm người tuyết",
                "Tìm và đến gần người tuyết đội bí ngô",
            ),
            Pipe => (
                "pipe",
                "Quái ống khói",
                "Nhìn vào đầu quái xanh rồi nhấp chuột ở gần",
            ),
            Mine => (
                "mine",
                "Bãi mìn ký ức",
                "Nhớ các ô đỏ; khi đèn tắt hãy tìm ô an toàn",
            ),
        }
    }
    pub fn scored(self) -> bool {
        matches!(
            self,
            Paint | Beacon | King | Treasure | Traffic | Snow | Pipe | Rally
        )
    }
}
pub struct Rng(u64);
impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed.max(1))
    }
    pub fn pick(&mut self, n: usize) -> usize {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        (x % (n as u64)) as usize
    }
}
pub fn cell(p: Pos) -> Option<usize> {
    if p.x < -9. || p.x >= 9. || p.z < -9. || p.z >= 9. {
        return None;
    }
    Some(((p.z.floor() as i32 + 9) * 18 + p.x.floor() as i32 + 9) as usize)
}
pub fn center(k: usize, y: f64) -> Pos {
    Pos::at((k % 18) as f64 - 8.5, y, (k / 18) as f64 - 8.5)
}
pub struct Runner {
    pub pos: Pos,
    pub alive: bool,
    pub score: i32,
    pub team: usize,
    pub hearts: i32,
    pub punch: bool,
    pub spawn: Pos,
    anchor: Pos,
    last_hit: u64,
}
impl Runner {
    pub fn new(i: usize, game: Game) -> Self {
        let k = if game == Traffic {
            17 * 18 + i % 18
        } else {
            (i * 53) % N
        };
        let p = if game == Rally {
            crate::course::spawn(i)
        } else {
            center(k, 73.)
        };
        Self {
            pos: p,
            alive: true,
            score: 0,
            team: i % 2,
            hearts: 3,
            punch: false,
            spawn: p,
            anchor: p,
            last_hit: 0,
        }
    }
}
#[derive(Debug)]
pub enum Action {
    Cue(String, String),
    Warp(i32, Pos),
    Out(i32),
    End(Vec<i32>),
}
pub struct Round {
    pub game: Game,
    pub floor: [i32; N],
    pub lower: [i32; N],
    pub props: [[i32; N]; 2],
    pub runners: BTreeMap<i32, Runner>,
    pub hint: String,
    pub wave: usize,
    pub end: u64,
    next: u64,
    phase: u8,
    target: usize,
    pattern: [i32; N],
    safe: [bool; N],
    due: [u64; N * 2],
    rng: Rng,
    step: usize,
    found: usize,
    initial: usize,
}
impl Round {
    pub fn new(game: Game, ids: &[i32], now: u64, seed: u64) -> Self {
        let mut s = Self {
            game,
            floor: [WHITE; N],
            lower: [0; N],
            props: [[0; N]; 2],
            runners: ids
                .iter()
                .enumerate()
                .map(|(i, &id)| (id, Runner::new(i, game)))
                .collect(),
            hint: String::new(),
            wave: 0,
            end: now + if game == Rally { 150000 } else { 60000 },
            next: now,
            phase: 0,
            target: 0,
            pattern: [0; N],
            safe: [false; N],
            due: [0; N * 2],
            rng: Rng::new(seed),
            step: 0,
            found: 0,
            initial: ids.len(),
        };
        match game {
            Tnt => {
                s.floor.fill(TNT);
                s.lower.fill(TNT)
            }
            Paint => s.hint = "Đỏ và Xanh tranh 324 ô".into(),
            Treasure => {
                while s.safe.iter().filter(|&&x| x).count() < 5 {
                    let i = s.rng.pick(N);
                    s.safe[i] = true
                }
                s.hint = "Chìa khóa 0/5".into()
            }
            Traffic => s.floor[..18].fill(GOLD),
            _ => (),
        }
        s
    }
    fn message(&mut self, main: String, sub: &str, actions: &mut Vec<Action>) {
        self.hint = main.clone();
        actions.push(Action::Cue(main, sub.to_owned()))
    }
    fn choose_pattern(&mut self) {
        let mut patches = [0; 36];
        for (i, x) in patches.iter_mut().enumerate() {
            *x = i % 6
        }
        for i in (1..36).rev() {
            patches.swap(i, self.rng.pick(i + 1))
        }
        for k in 0..N {
            self.pattern[k] = DYES[patches[(k / 18 / 3) * 6 + k % 18 / 3]]
        }
    }
    fn judge(&mut self, a: &mut Vec<Action>) {
        for (&id, p) in &mut self.runners {
            if p.alive
                && !cell(p.pos).is_some_and(|k| self.safe[k] && (72.7..76.).contains(&p.pos.y))
            {
                p.alive = false;
                a.push(Action::Out(id))
            }
        }
    }
    fn winners(&self) -> Vec<i32> {
        if self.game == Treasure {
            return self
                .runners
                .keys()
                .copied()
                .filter(|_| self.found == 5)
                .collect();
        }
        if self.game == Paint {
            let scores = [RED, BLUE].map(|d| self.floor.iter().filter(|&&x| x == d).count());
            return self
                .runners
                .iter()
                .filter(|(_, p)| scores[p.team] > 0 && scores[p.team] >= scores[1 - p.team])
                .map(|(&i, _)| i)
                .collect();
        }
        if self.game.scored() {
            let high = self.runners.values().map(|p| p.score).max().unwrap_or(0);
            return self
                .runners
                .iter()
                .filter(|(_, p)| high > 0 && p.score == high)
                .map(|(&i, _)| i)
                .collect();
        }
        self.runners
            .iter()
            .filter(|(_, p)| p.alive)
            .map(|(&i, _)| i)
            .collect()
    }
    pub fn update(&mut self, now: u64) -> Vec<Action> {
        let mut a = vec![];
        if now >= self.end || self.runners.is_empty() {
            return vec![Action::End(self.winners())];
        }
        if self.game == Rally {
            use crate::course::GATES;
            let mut finishers = Vec::new();
            for (&id, p) in &mut self.runners {
                if !(-30.0..50.0).contains(&p.pos.x)
                    || !(-30.0..37.0).contains(&p.pos.z)
                    || !(88.8..102.).contains(&p.pos.y)
                {
                    p.pos = p.anchor;
                    a.push(Action::Warp(id, p.anchor));
                    continue;
                }
                let (x, z, _) = GATES[p.score as usize];
                if (p.pos.x - x as f64 - 0.5).hypot(p.pos.z - z as f64 - 0.5) <= 2.5
                    && p.pos.y <= 91.0
                {
                    p.score += 1;
                    p.anchor = Pos::at(x as f64 + 0.5, 89., z as f64 + 0.5);
                    if p.score as usize == GATES.len() {
                        finishers.push(id);
                    }
                }
            }
            if !finishers.is_empty() {
                a.push(Action::End(finishers));
            }
            return a;
        }
        for (&id, p) in &mut self.runners {
            if !p.alive {
                continue;
            }
            if cell(p.pos).is_none()
                || p.pos.y < if self.game == Tnt { 64. } else { 69. }
                || p.pos.y > 80.
            {
                if self.game.scored() {
                    p.pos = p.spawn;
                    p.anchor = p.spawn;
                    a.push(Action::Warp(id, p.spawn))
                } else {
                    p.alive = false;
                    a.push(Action::Out(id))
                }
            }
        }
        match self.game {
            Rally => unreachable!(),
            Color | Memory | Islands | Mine => {
                if now >= self.next {
                    self.puzzle(now, &mut a)
                }
            }
            Snow | Pipe => {
                const POSTS: [usize; 9] = [38, 45, 52, 146, 153, 160, 254, 261, 268];
                if now >= self.next {
                    let previous = self.target;
                    loop {
                        self.target = POSTS[self.rng.pick(POSTS.len())];
                        if self.target != previous {
                            break;
                        }
                    }
                    for &k in &POSTS {
                        self.props[0][k] = if self.game == Snow { 8598 } else { 3832 };
                        self.props[1][k] = if k == self.target {
                            if self.game == Snow {
                                8689
                            } else {
                                12858
                            }
                        } else if self.game == Snow {
                            8598
                        } else {
                            0
                        };
                    }
                    self.next = now + if self.game == Snow { 7500 } else { 3700 };
                    self.hint = if self.game == Snow {
                        "Tìm đầu bí ngô!".into()
                    } else {
                        "Ngắm đầu quái rồi bấm chuột!".into()
                    };
                }
                let t = center(self.target, 74.5);
                let mut hits = vec![];
                for (&id, p) in &mut self.runners {
                    let dx = t.x - p.pos.x;
                    let dz = t.z - p.pos.z;
                    let hit = if self.game == Snow {
                        dx.hypot(dz) < 1.9
                    } else {
                        let dy = t.y - p.pos.y - 1.62;
                        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
                        let yaw = (p.pos.yaw as f64).to_radians();
                        let pitch = (p.pos.pitch as f64).to_radians();
                        let dot = (-yaw.sin() * pitch.cos() * dx - pitch.sin() * dy
                            + yaw.cos() * pitch.cos() * dz)
                            / dist.max(0.01);
                        p.punch && dist < 4.5 && dot > 0.92
                    };
                    if hit {
                        hits.push(id)
                    }
                    p.punch = false;
                }
                if !hits.is_empty() {
                    for id in hits {
                        self.runners.get_mut(&id).unwrap().score += 1
                    }
                    self.props[1][self.target] = 0;
                    self.next = now;
                }
            }
            Tnt => {
                for p in self.runners.values().filter(|p| p.alive) {
                    if let Some(k) = cell(p.pos) {
                        let layer = if (p.pos.y - 73.).abs() < 0.08 {
                            0
                        } else if (p.pos.y - 67.).abs() < 0.08 {
                            1
                        } else {
                            continue;
                        };
                        let floor = if layer == 0 {
                            &mut self.floor
                        } else {
                            &mut self.lower
                        };
                        if floor[k] == TNT {
                            floor[k] = RED;
                            self.due[layer * N + k] = now + 400
                        }
                    }
                }
                for (i, t) in self.due.iter_mut().enumerate() {
                    if *t > 0 && now >= *t {
                        if i < N {
                            self.floor[i] = 0
                        } else {
                            self.lower[i - N] = 0
                        }
                        *t = 0
                    }
                }
            }
            Wave => {
                if now >= self.next {
                    self.next = now + if self.wave < 3 { 350 } else { 250 };
                    self.step %= 18;
                    if self.step == 0 {
                        self.wave += 1;
                        self.target = self.rng.pick(14);
                        self.message(format!("Sóng {}", self.wave), "Chạy vào khe trống", &mut a)
                    }
                    self.floor.fill(WHITE);
                    for k in 0..N {
                        let (x, z) = (k % 18, k / 18);
                        let (line, cross) = if self.wave % 2 == 1 { (z, x) } else { (x, z) };
                        if line == self.step && !(self.target..self.target + 4).contains(&cross) {
                            self.floor[k] = RED
                        }
                    }
                    for (&id, p) in &mut self.runners {
                        if p.alive
                            && p.last_hit != self.wave as u64
                            && cell(p.pos).is_some_and(|k| self.floor[k] == RED)
                        {
                            p.last_hit = self.wave as u64;
                            p.hearts -= 1;
                            if p.hearts == 0 {
                                p.alive = false;
                                a.push(Action::Out(id))
                            }
                        }
                    }
                    self.step += 1
                }
            }
            Paint => {
                for p in self.runners.values().filter(|p| p.alive) {
                    if let Some(k) = cell(p.pos) {
                        self.floor[k] = [RED, BLUE][p.team]
                    }
                }
                if now >= self.next {
                    self.next = now + 500;
                    self.hint = format!(
                        "Đỏ {} · Xanh {}",
                        self.floor.iter().filter(|&&d| d == RED).count(),
                        self.floor.iter().filter(|&&d| d == BLUE).count()
                    )
                }
            }
            Treasure => {
                for p in self.runners.values_mut().filter(|p| p.alive) {
                    if let Some(k) = cell(p.pos) {
                        if self.floor[k] == WHITE {
                            if self.safe[k] {
                                self.floor[k] = GOLD;
                                self.found += 1;
                                p.score += 1;
                                a.push(Action::Cue(
                                    format!("Chìa khóa {}/5", self.found),
                                    "Cả đội cùng tìm!".into(),
                                ))
                            } else {
                                let near = (0..N).any(|j| {
                                    self.safe[j]
                                        && (j % 18).abs_diff(k % 18) + (j / 18).abs_diff(k / 18)
                                            <= 2
                                });
                                self.floor[k] = if near { LIME } else { BLUE }
                            }
                        }
                    }
                }
                self.hint = format!("Chìa khóa {}/5 · xanh lá = gần", self.found);
                if self.found == 5 {
                    a.push(Action::End(self.winners()));
                    return a;
                }
            }
            Beacon => {
                if now >= self.next {
                    self.floor.fill(WHITE);
                    self.target = self.rng.pick(N);
                    self.floor[self.target] = GOLD;
                    self.next = now + 5000;
                    self.hint = "Chạm đốm vàng!".into()
                }
                let hits: Vec<_> = self
                    .runners
                    .iter()
                    .filter(|(_, p)| p.alive && cell(p.pos) == Some(self.target))
                    .map(|(&id, _)| id)
                    .collect();
                if !hits.is_empty() {
                    for id in hits {
                        self.runners.get_mut(&id).unwrap().score += 1
                    }
                    self.next = now
                }
            }
            King => {
                if now >= self.next {
                    self.floor.fill(WHITE);
                    let x = self.rng.pick(14);
                    let z = self.rng.pick(14);
                    for dz in 0..5 {
                        for dx in 0..5 {
                            self.floor[(z + dz) * 18 + x + dx] = LIME
                        }
                    }
                    self.next = now + 7000;
                    self.message(
                        "Đài đã chuyển!".into(),
                        "Đứng trên ô xanh để tích điểm",
                        &mut a,
                    )
                }
                if now / 1000 != self.step as u64 {
                    self.step = (now / 1000) as usize;
                    for p in self.runners.values_mut() {
                        if cell(p.pos).is_some_and(|k| self.floor[k] == LIME) {
                            p.score += 1
                        }
                    }
                }
            }
            Traffic => {
                if now >= self.next {
                    self.phase = 1 - self.phase;
                    let green = self.phase == 1;
                    self.next = now
                        + if green {
                            2800 + self.rng.pick(1800) as u64
                        } else {
                            1800
                        };
                    self.step = (now + 350) as usize;
                    self.floor[18..].fill(if green { LIME } else { RED });
                    self.message(
                        if green { "ĐÈN XANH" } else { "ĐÈN ĐỎ" }.into(),
                        "Tới vạch vàng phía bắc",
                        &mut a,
                    )
                }
                for (&id, p) in &mut self.runners {
                    if self.phase == 1 || now < self.step as u64 {
                        p.anchor = p.pos
                    } else if (p.pos.x - p.anchor.x).hypot(p.pos.z - p.anchor.z) > 0.35 {
                        p.pos = p.spawn;
                        p.anchor = p.spawn;
                        a.push(Action::Warp(id, p.spawn));
                        continue;
                    }
                    if p.pos.z < -8.0 {
                        p.score += 1;
                        p.pos = p.spawn;
                        p.anchor = p.spawn;
                        a.push(Action::Warp(id, p.spawn))
                    }
                }
            }
            Ice => {
                if now >= self.next {
                    self.wave += 1;
                    self.next = now + 900;
                    for _ in 0..(4 + self.wave.min(14)) {
                        let k = self.rng.pick(N);
                        if self.floor[k] == WHITE {
                            self.floor[k] = RED;
                            self.due[k] = now + 850
                        }
                    }
                    self.hint = format!("Băng vỡ · đợt {}", self.wave)
                }
                for k in 0..N {
                    if self.due[k] > 0 && now >= self.due[k] {
                        self.floor[k] = 0;
                        self.due[k] = 0
                    }
                }
            }
        }
        if !self.game.scored() {
            let n = self.runners.values().filter(|p| p.alive).count();
            if n == 0 || (self.initial > 1 && n == 1) {
                a.push(Action::End(self.winners()))
            }
        }
        a
    }
    fn puzzle(&mut self, now: u64, a: &mut Vec<Action>) {
        if self.phase == 0 {
            self.wave += 1;
            if self.wave > 10 {
                a.push(Action::End(self.winners()));
                return;
            }
            self.choose_pattern();
            self.target = self.rng.pick(6);
            self.safe.fill(false);
            for k in 0..N {
                self.safe[k] = self.pattern[k] == DYES[self.target]
            }
            match self.game {
                Memory => {
                    self.floor = self.pattern;
                    self.message(
                        "Ghi nhớ vị trí!".into(),
                        "Màu đích sẽ xuất hiện sau khi sàn trắng",
                        a,
                    );
                    self.next = now + 2500;
                    self.phase = 1
                }
                Mine => {
                    for k in 0..N {
                        self.safe[k] = self.rng.pick(100) > 30 + self.wave * 2;
                        self.floor[k] = if self.safe[k] { WHITE } else { RED }
                    }
                    self.safe[0] = true;
                    self.floor[0] = WHITE;
                    self.message("Nhớ các ô mìn đỏ!".into(), "Chọn ô không có mìn", a);
                    self.next = now + 2000;
                    self.phase = 1
                }
                Islands => {
                    let side = 7 - self.wave.min(5);
                    let x = self.rng.pick(19 - side);
                    let z = self.rng.pick(19 - side);
                    for k in 0..N {
                        self.safe[k] =
                            (x..x + side).contains(&(k % 18)) && (z..z + side).contains(&(k / 18));
                        self.floor[k] = if self.safe[k] { LIME } else { BLUE }
                    }
                    self.message("Nước đang dâng!".into(), "Chạy đến đảo xanh lá", a);
                    self.next = now + 4500;
                    self.phase = 2
                }
                _ => {
                    self.floor = self.pattern;
                    self.message(NAMES[self.target].into(), "Đứng trên màu này!", a);
                    self.next = now + (6000 - 350 * self.wave as u64);
                    self.phase = 2
                }
            }
        } else if self.phase == 1 {
            self.floor.fill(WHITE);
            self.message(
                if self.game == Memory {
                    NAMES[self.target]
                } else {
                    "Tránh ô mìn!"
                }
                .into(),
                "Chốt vị trí trước khi hết giờ",
                a,
            );
            self.next = now + 3200;
            self.phase = 2
        } else {
            for k in 0..N {
                self.floor[k] = if self.safe[k] { LIME } else { 0 }
            }
            self.judge(a);
            self.phase = 0;
            self.next = now + 1500
        }
    }
}
#[cfg(test)]
#[path = "game_tests.rs"]
mod tests;
