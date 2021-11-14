
use crate::protos::module::{SOILModuleData};

pub struct AAPValidator {}

impl super::interface::ModuleValue for SOILModuleData {}

impl super::interface::ModuleValueValidator<SOILModuleData> for AAPValidator {

    fn convert_to_value(&self, value_event: &crate::comboard::imple::interface::ModuleValueValidationEvent) -> SOILModuleData {
        let mut data = SOILModuleData::new();
        data.p0 = value_event.buffer[0] as i32;
        data.p1 = value_event.buffer[1] as i32;
        data.p2 = value_event.buffer[2] as i32;
        data.p3 = value_event.buffer[3] as i32;
        data.p4 = value_event.buffer[4] as i32;
        data.p5 = value_event.buffer[5] as i32;
        data.p6 = value_event.buffer[6] as i32;
        data.p7 = value_event.buffer[7] as i32;

        return data;
    }

}