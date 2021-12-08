
use protobuf::SingularPtrField;

pub fn difference_of<T: std::ops::Sub<Output = T>  + std::cmp::PartialOrd + std::ops::Add<Output = T> + Copy>(current: T, past: T, gap: T) -> bool {
    return (past + gap <= current) || (past - gap >= current)
}
