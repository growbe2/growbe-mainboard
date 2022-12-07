use super::model::{ModuleAlarmState, ModuleValueChange};
use crate::modulestate::interface::ModuleError;
use crate::protos::alarm::{AlarmZone, AlarmZoneValue, FieldAlarm, FieldAlarmEvent};

use std::collections::HashMap;

pub struct StoreAlarmItem {
    pub field_alarm: FieldAlarm,
    pub state: ModuleAlarmState<f32>,
}

// AlarmFieldValidator is the class that validate
// the alarm state and trigger alarm when needed
pub struct AlarmFieldValidator {
    pub maps: HashMap<String, StoreAlarmItem>,
}

impl AlarmFieldValidator {
    pub fn new() -> Self {
        return AlarmFieldValidator {
            maps: HashMap::new(),
        };
    }

    fn get_value_zone(
        current: f32,
        low: &AlarmZoneValue,
        high: &AlarmZoneValue,
        current_zone: AlarmZone,
    ) -> AlarmZone {
        let low_alarm = low.value;
        let low_offset = low.offset;
        let high_alarm = high.value;
        let high_offset = high.offset;

        if current_zone == AlarmZone::MIDDLE {
            if current >= high_alarm + high_offset {
                return AlarmZone::HIGH;
            } else if current <= low_alarm - low_offset {
                return AlarmZone::LOW;
            }
        } else {
            if current_zone == AlarmZone::HIGH && current <= high_alarm - high_offset {
                return AlarmZone::MIDDLE;
            } else if current_zone == AlarmZone::LOW && current >= low_alarm + low_offset {
                return AlarmZone::MIDDLE;
            } else if current_zone == AlarmZone::UNKNOW {
                if current >= high_alarm {
                    return AlarmZone::HIGH;
                } else if current <= low_alarm {
                    return AlarmZone::LOW;
                }
                return AlarmZone::MIDDLE;
            }
        }

        return AlarmZone::UNKNOW;
    }

    pub fn deregister_field_alarm(&mut self, alarm: FieldAlarm) -> Result<(), ModuleError> {
        let id = self.get_id(&alarm.moduleId, &alarm.property);
        match self.maps.remove(&id) {
            Some(_v) => {
                log::info!("deregistering alarm on {}", id.as_str());
                Ok(())
            }
            None => {
                // Err(ModuleError::new());
                Ok(())
            }
        }
    }

    pub fn register_field_alarm(
        &mut self,
        alarm: FieldAlarm,
        state: Option<super::model::ModuleAlarmState<f32>>,
    ) -> Result<(), ModuleError> {
        let id = self.get_id(&alarm.moduleId, &alarm.property);

        log::info!("registering alarm on {}", id.as_str());

        let state = if let Some(state) = state {
            state
        } else {
            ModuleAlarmState::<f32> {
                property: alarm.property.clone(),
                current_value: 0.,
                previous_value: 0.,
                last_diff: 0.,
                zone: crate::protos::alarm::AlarmZone::UNKNOW,
            }
        };

        self.maps.insert(
            id,
            StoreAlarmItem {
                state,
                field_alarm: alarm,
            },
        );

        Ok(())
    }

    pub fn on_module_value_change(
        &mut self,
        change: &ModuleValueChange<f32>,
    ) -> Vec<(FieldAlarmEvent, ModuleAlarmState<f32>)> {
        log::info!(
            "Got change {:?} {:?}",
            change.module_id.as_str(),
            change.changes.len()
        );

        return change
            .changes
            .iter()
            .filter_map(|value| {
                let result = self.get_store_item(&change.module_id, &value.property);
                if let Some(item) = result {
                    let new_zone = AlarmFieldValidator::get_value_zone(
                        value.current_value,
                        item.field_alarm.get_low(),
                        item.field_alarm.get_high(),
                        item.state.zone,
                    );

                    if new_zone != AlarmZone::UNKNOW && new_zone != item.state.zone {
                        let mut event = FieldAlarmEvent::new();
                        event.property = value.property.clone();
                        event.previousValue = item.state.previous_value;
                        event.previousZone = item.state.zone;
                        event.currentValue = value.current_value;
                        event.currentZone = new_zone;
                        event.moduleId = change.module_id.clone();

                        item.state.current_value = value.current_value;
                        item.state.zone = new_zone;

                        return Some((event, item.state.clone()));
                    }

                    item.state.previous_value = value.current_value;
                }
                return None;
            })
            .collect();
    }

    fn get_store_item(
        &mut self,
        module_id: &String,
        property: &String,
    ) -> Option<&mut StoreAlarmItem> {
        let id = self.get_id(module_id, property);
        self.maps.get_mut(&id)
    }

    fn get_id(&self, module_id: &String, property: &String) -> String {
        let mut id = module_id.clone();
        id.push_str(":");
        id.push_str(property.as_str());
        id
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::protos::alarm::{AlarmZoneValue, FieldAlarm};

    fn get_value_change(current: f32) -> ModuleValueChange<f32> {
        ModuleValueChange::<f32> {
            module_id: "ABC".to_string(),
            changes: vec![crate::modulestate::alarm::model::ValueChange::<f32> {
                property: "p0".to_string(),
                current_value: current,
                previous_value: 0.,
            }],
        }
    }

    fn get_alarm(low_value: f32, high_value: f32, offset: f32) -> FieldAlarm {
        let mut alarm = FieldAlarm::new();
        alarm.moduleId = "ABC".to_string();
        alarm.property = "p0".to_string();

        let mut low = AlarmZoneValue::new();
        low.value = low_value;
        low.offset = offset;
        alarm.low = protobuf::SingularPtrField::some(low);

        let mut high = AlarmZoneValue::new();
        high.value = high_value;
        high.offset = offset;
        alarm.high = protobuf::SingularPtrField::some(high);

        alarm
    }

    #[test]
    fn value_change_when_no_field_register_does_nothing() {
        let mut validator = AlarmFieldValidator::new();

        let change = get_value_change(0.);

        let events = validator.on_module_value_change(&change);

        assert_eq!(events.len(), 0);
    }

    #[test]
    fn value_change_high_no_multiple_trigger() {
        let mut validator = AlarmFieldValidator::new();

        let alarm_field = get_alarm(30., 70., 3.);

        let values: Vec<f32> = vec![
            30., 30., 32., 40., 50., 60., 70., 71., 72., 73., 72., 69., 70., 69., 70., 69., 68.,
            68., 67., 50., 50.,
        ];

        validator.register_field_alarm(alarm_field, None).unwrap();

        let events: Vec<FieldAlarmEvent> = values
            .iter()
            .map(|x| validator.on_module_value_change(&get_value_change(*x)))
            .flat_map(|x| x)
            .map(|x| x.0)
            .collect();

        assert_eq!(events.len(), 4);

        assert_eq!(events[0].currentZone, AlarmZone::LOW);
        assert_eq!(events[0].previousValue, 0.);
        assert_eq!(events[0].currentValue, 30.);

        assert_eq!(events[1].currentZone, AlarmZone::MIDDLE);
        assert_eq!(events[1].previousValue, 32.);
        assert_eq!(events[1].currentValue, 40.);

        assert_eq!(events[2].currentZone, AlarmZone::HIGH);
        assert_eq!(events[2].previousValue, 72.);
        assert_eq!(events[2].currentValue, 73.);

        assert_eq!(events[3].currentZone, AlarmZone::MIDDLE);
    }

    #[test]
    fn value_change_low_no_multiple_trigger() {
        let mut validator = AlarmFieldValidator::new();

        let alarm_field = get_alarm(30., 70., 3.);

        let values: Vec<f32> = vec![
            50., 50., 44., 54., 23., 53., 31., 31., 30., 29., 28., 28., 28., 27., 26., 26., 24.,
            30., 31., 34.,
        ];

        validator.register_field_alarm(alarm_field, None).unwrap();

        let events: Vec<FieldAlarmEvent> = values
            .iter()
            .map(|x| validator.on_module_value_change(&get_value_change(*x)))
            .flat_map(|x| x)
            .map(|x| x.0)
            .collect();

        assert_eq!(events.len(), 5);
        assert_eq!(events[0].currentZone, AlarmZone::MIDDLE);
        assert_eq!(events[1].currentZone, AlarmZone::LOW);
        assert_eq!(events[2].currentZone, AlarmZone::MIDDLE);
        assert_eq!(events[3].currentZone, AlarmZone::LOW);
        assert_eq!(events[4].currentZone, AlarmZone::MIDDLE);
    }
}
