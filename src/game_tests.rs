use super::*;
fn round(game: Game) -> Round {
    Round::new(game, &[1, 2], 0, 12345)
}
fn stand(r: &mut Round, id: i32, k: usize) {
    r.runners.get_mut(&id).unwrap().pos = center(k, 73.)
}
#[test]
fn all_modes_finish() {
    for g in GAMES {
        let mut r = round(g);
        assert!(matches!(r.update(r.end).as_slice(), [Action::End(_)]))
    }
}
#[test]
fn puzzles_judge_positions() {
    for g in [Color, Memory, Islands, Mine] {
        let mut r = round(g);
        r.update(0);
        if r.phase == 1 {
            r.update(r.next);
        }
        let safe = r.safe.iter().position(|&s| s).unwrap();
        let bad = r.safe.iter().position(|&s| !s).unwrap();
        stand(&mut r, 1, safe);
        stand(&mut r, 2, bad);
        let a = r.update(r.next);
        assert!(
            a.iter().any(|a| matches!(a,Action::End(w) if w==&vec![1])),
            "{g:?}"
        )
    }
}
#[test]
fn tnt_both_layers() {
    let mut r = round(Tnt);
    stand(&mut r, 1, 10);
    r.update(0);
    assert_eq!(r.floor[10], RED);
    r.update(400);
    assert_eq!(r.floor[10], 0);
    r.runners.get_mut(&1).unwrap().pos = center(10, 67.);
    r.update(500);
    r.update(900);
    assert_eq!(r.lower[10], 0)
}
#[test]
fn paint_changes_owner() {
    let mut r = round(Paint);
    stand(&mut r, 1, 2);
    stand(&mut r, 2, 3);
    r.update(0);
    stand(&mut r, 1, 4);
    stand(&mut r, 2, 2);
    r.update(50);
    assert_eq!(r.floor[2], BLUE);
    assert_eq!(r.winners(), vec![2])
}
#[test]
fn find_all_keys() {
    let mut r = round(Treasure);
    let keys: Vec<_> = (0..N).filter(|&k| r.safe[k]).collect();
    let mut last = vec![];
    for k in keys {
        stand(&mut r, 1, k);
        last = r.update(20)
    }
    assert_eq!(r.found, 5);
    assert!(last
        .iter()
        .any(|a| matches!(a,Action::End(w) if w.len()==2)))
}
#[test]
fn wave_takes_one_heart_per_pass() {
    let mut r = round(Wave);
    stand(&mut r, 1, 170);
    r.update(0);
    let k = (0..18).find(|&k| r.floor[k] == RED).unwrap() + 18;
    stand(&mut r, 1, k);
    let hp = r.runners[&1].hearts;
    r.update(350);
    assert_eq!(r.runners[&1].hearts, hp - 1);
    r.update(360);
    assert_eq!(r.runners[&1].hearts, hp - 1)
}
#[test]
fn beacon_scores() {
    let mut r = round(Beacon);
    r.update(0);
    let k = r.target;
    stand(&mut r, 1, k);
    r.update(100);
    assert!(r.runners[&1].score > 0)
}
#[test]
fn king_scores_in_zone() {
    let mut r = round(King);
    r.update(0);
    let k = r.floor.iter().position(|&b| b == LIME).unwrap();
    stand(&mut r, 1, k);
    r.update(1000);
    assert_eq!(r.runners[&1].score, 1)
}
#[test]
fn traffic_red_and_finish() {
    let mut r = round(Traffic);
    r.update(0);
    stand(&mut r, 1, 0);
    r.update(100);
    assert_eq!(r.runners[&1].score, 1);
    let red = r.next;
    r.update(red);
    stand(&mut r, 1, 100);
    assert!(r
        .update(red + 500)
        .iter()
        .any(|a| matches!(a, Action::Warp(1, _))))
}
#[test]
fn ice_collapses() {
    let mut r = round(Ice);
    r.update(0);
    let k = r.floor.iter().position(|&x| x == RED).unwrap();
    r.update(850);
    assert_eq!(r.floor[k], 0)
}
#[test]
fn winter_models_and_hits() {
    for g in [Snow, Pipe] {
        let mut r = round(g);
        r.update(0);
        let k = r.target;
        let mut pos = center(k, 73.);
        pos.z -= 1.5;
        pos.yaw = 0.;
        let p = r.runners.get_mut(&1).unwrap();
        p.pos = pos;
        p.punch = true;
        let score = p.score;
        r.update(50);
        assert!(r.runners[&1].score > score, "{g:?}")
    }
}
