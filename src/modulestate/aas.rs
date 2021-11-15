use crate::protos::module::{THLModuleData};

pub struct AASValidator {}

impl super::interface::ModuleValue for THLModuleData {}

impl super::interface::ModuleValueParsable for THLModuleData {}

impl super::interface::ModuleValueValidator for AASValidator {
    fn convert_to_value(&self, value_event: &crate::comboard::imple::interface::ModuleValueValidationEvent) -> Box<dyn super::interface::ModuleValueParsable> {
        let mut data = THLModuleData::new();
        data.airTemperature = value_event.buffer[0] as f32;
        data.humidity = value_event.buffer[1] as f32;
        return Box::new(data);
    }

}