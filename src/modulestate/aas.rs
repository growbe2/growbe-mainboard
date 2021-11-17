
use crate::protos::module::{SOILModuleData};

pub struct AASValidator {}

impl super::interface::ModuleValue for SOILModuleData {}

impl super::interface::ModuleValueParsable for SOILModuleData {}

impl super::interface::ModuleValueValidator for AASValidator {

    fn convert_to_value(&self, value_event: &crate::comboard::imple::interface::ModuleValueValidationEvent) -> Box<dyn super::interface::ModuleValueParsable> {
        let mut data = SOILModuleData::new();
        data.p0 = value_event.buffer[0] as i32;
        data.p1 = value_event.buffer[1] as i32;
        data.p2 = value_event.buffer[2] as i32;
        data.p3 = value_event.buffer[3] as i32;
        data.p4 = value_event.buffer[4] as i32;
        data.p5 = value_event.buffer[5] as i32;
        data.p6 = value_event.buffer[6] as i32;
        data.p7 = value_event.buffer[7] as i32;

        return Box::new(data);
    }

    
    fn apply_parse_config(&self, port: i32, t: char, data: std::sync::Arc<Vec<u8>>) -> (Box<dyn protobuf::Message>, crate::comboard::imple::interface::Module_Config) {
        panic!("AAS has no config");
    }

}