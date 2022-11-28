pub fn difference_of<
    T: std::ops::Sub<Output = T> + std::cmp::PartialOrd + std::ops::Add<Output = T> + Copy,
>(
    current: T,
    past: T,
    gap: T,
) -> bool {
    return (past + gap <= current) || (past - gap >= current);
}

// make generic plz and accept custom digit
pub fn round_decimal(x: f32) -> f32 {
    return (x * 100.0).round() / 100.0;
}
