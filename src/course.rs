use crate::wire::Pos;
pub const START: (i32, i32) = (-22, -20);
pub const GATES: [(i32, i32, &str); 6] = [
    (-22, 16, "Rừng thông"),
    (8, 16, "Bến thuyền"),
    (8, -16, "Cầu hồ"),
    (39, -16, "Tháp canh"),
    (39, 27, "Đèo tuyết"),
    (-5, 27, "Khải hoàn"),
];
pub fn spawn(i: usize) -> Pos {
    Pos::at(
        (START.0 + (i % 5) as i32 - 2) as f64 + 0.5,
        89.,
        (START.1 - (i / 5 % 5) as i32) as f64 + 0.5,
    )
}
pub fn balcony() -> Pos {
    Pos::at(19.5, 99., 5.5)
}
