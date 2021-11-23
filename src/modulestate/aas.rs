
use crate::protos::module::{SOILModuleData};

pub struct AASValidator {}

impl super::interface::ModuleValue for SOILModuleData {}

impl super::interface::ModuleValueParsable for SOILModuleData {}

impl super::interface::ModuleValueValidator for AASValidator {

    fn convert_to_value(&self, value_event: &crate::comboard::imple::interface::ModuleValueValidationEvent) -> Box<dyn super::interface::ModuleValueParsable> {
        let mut data = SOILModuleData::new();

        data.p0 = value_event.buffer[0] as i32;
        data.p1 = value_event.buffer[50] as i32;
        data.p2 = value_event.buffer[100] as i32;
        data.p3 = value_event.buffer[150] as i32;
        data.p4 = value_event.buffer[200] as i32;
        data.p5 = value_event.buffer[250] as i32;
        data.p6 = value_event.buffer[300] as i32;
        data.p7 = value_event.buffer[350] as i32;

        return Box::new(data);
    }
    
    fn apply_parse_config(&self, port: i32, t: char, data: std::sync::Arc<Vec<u8>>, sender_comboard_config: & std::sync::mpsc::Sender<crate::comboard::imple::interface::Module_Config>,
        map_handler: & mut std::collections::HashMap<i32, tokio::task::JoinHandle<()>>
    ) -> (Box<dyn protobuf::Message>, crate::comboard::imple::interface::Module_Config) {
        panic!("AAS has no config");
    }

    fn have_data_change(&self, current: &Box<dyn crate::modulestate::interface::ModuleValueParsable>, last: &Box<dyn crate::modulestate::interface::ModuleValueParsable>) -> bool {
        let it: &dyn std::any::Any = current.as_any();
        let current = match it.downcast_ref::<SOILModuleData>() {
            Some(i) => i,
            None => panic!(),
        };

        let it: &dyn std::any::Any = last.as_any();
        let current = match it.downcast_ref::<SOILModuleData>() {
            Some(i) => i,
            None => panic!(),
        };

        return true;
    }
}