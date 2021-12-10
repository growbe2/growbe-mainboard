use crate::protos::alarm::AlarmZone;

pub struct ModuleAlarmState<T: std::ops::Sub<Output = T>  + std::cmp::PartialOrd + std::ops::Add<Output = T> + Copy> {
    pub property: String,
    pub current_value: T,
    pub previous_value: T,
    pub last_diff: T,
    pub zone: AlarmZone,
}

// Structure send by the modulestate task to tell that we have change on the value of a module
// and we need to valide them
pub struct ModuleValueChange<T: std::ops::Sub<Output = T>  + std::cmp::PartialOrd + std::ops::Add<Output = T> + Copy> {
    pub module_id: String,
    pub changes: Vec<ValueChange<T>>,
}

pub struct ValueChange<T: std::ops::Sub<Output = T>  + std::cmp::PartialOrd + std::ops::Add<Output = T> + Copy> {
    pub property: String,
    pub current_value: T,
    pub previous_value: T,
}