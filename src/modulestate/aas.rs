use crate::protos::module::{THLModuleData};

pub struct AASValidator {}

impl super::interface::ModuleValue for THLModuleData {}

impl super::interface::ModuleValueValidator<THLModuleData> for AASValidator {

    fn convert_to_value(&self, value_event: &crate::comboard::imple::interface::ModuleValueValidationEvent) -> THLModuleData {
        let mut data = THLModuleData::new();
        data.airTemperature = value_event.buffer[0] as i32;
        data.humidity = value_event.buffer[1] as i32;
        return data;
    }

}