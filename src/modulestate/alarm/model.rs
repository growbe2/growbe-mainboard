use crate::protos::alarm::AlarmZone;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(remote = "AlarmZone")]
#[allow(non_camel_case_types)]
enum AlarmZoneDef {
    UNKNOW = 0,
    MIDDLE = 1,
    VERY_LOW = 2,
    LOW = 3,
    HIGH = 4,
    VERY_HIGH = 5,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ModuleAlarmState<
    T: std::ops::Sub<Output = T> + std::cmp::PartialOrd + std::ops::Add<Output = T> + Copy,
> {
    pub property: String,
    pub current_value: T,
    pub previous_value: T,
    pub last_diff: T,
    #[serde(with = "AlarmZoneDef")]
    pub zone: AlarmZone,
}

impl Default for ModuleAlarmState<f32> {
    fn default() -> Self {
        return Self {
            property: "".to_string(),
            current_value: 0.,
            previous_value: 0.,
            last_diff: 0.,
            zone: AlarmZone::UNKNOW,
        };
    }
}

// Structure send by the modulestate task to tell that we have change on the value of a module
// and we need to valide them
pub struct ModuleValueChange<
    T: std::ops::Sub<Output = T> + std::cmp::PartialOrd + std::ops::Add<Output = T> + Copy,
> {
    pub module_id: String,
    pub changes: Vec<ValueChange<T>>,
}

pub struct ValueChange<
    T: std::ops::Sub<Output = T> + std::cmp::PartialOrd + std::ops::Add<Output = T> + Copy,
> {
    pub property: String,
    pub current_value: T,
    pub previous_value: T,
}

impl crate::modulestate::interface::ModuleValue for crate::protos::alarm::FieldAlarmEvent {}

impl crate::modulestate::interface::ModuleValueParsable for crate::protos::alarm::FieldAlarmEvent {}

impl crate::protos::alarm::FieldAlarmEvent {
    pub fn clone_me(&self) -> crate::protos::alarm::FieldAlarmEvent {
        let mut new = crate::protos::alarm::FieldAlarmEvent::new();
        new.currentValue = self.currentValue;
        new.currentZone = self.currentZone;
        new.moduleId = self.moduleId.clone();
        new.previousValue = self.previousValue;
        new.previousZone = self.previousZone;
        new.property = self.property.clone();

        new
    }
}
