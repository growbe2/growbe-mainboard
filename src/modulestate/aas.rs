
use crate::protos::module::{SOILModuleData};
use crate::utils::validation::difference_of;

pub struct AASValidator {}

impl super::interface::ModuleValue for SOILModuleData {}

impl super::interface::ModuleValueParsable for SOILModuleData {}

impl super::interface::ModuleValueValidator for AASValidator {

    fn convert_to_value(&self, value_event: &crate::comboard::imple::interface::ModuleValueValidationEvent) -> Result<Box<dyn super::interface::ModuleValueParsable>, super::interface::ModuleError> {
        let mut data = SOILModuleData::new();

        if value_event.buffer.len() > 350 {
            data.p0 = value_event.buffer[0] as i32;
            data.p1 = value_event.buffer[50] as i32;
            data.p2 = value_event.buffer[100] as i32;
            data.p3 = value_event.buffer[150] as i32;
            data.p4 = value_event.buffer[200] as i32;
            data.p5 = value_event.buffer[250] as i32;
            data.p6 = value_event.buffer[300] as i32;
            data.p7 = value_event.buffer[350] as i32;
            data.timestamp = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_secs() as i32;
        }

        return Ok(Box::new(data));
    }
    
    fn apply_parse_config(&self, _port: i32, _t: char, _data: std::sync::Arc<Vec<u8>>, _sender_comboard_config: & std::sync::mpsc::Sender<crate::comboard::imple::interface::Module_Config>,
        _map_handler: & mut std::collections::HashMap<i32, tokio::task::JoinHandle<()>>
    ) -> Result<(Box<dyn protobuf::Message>, crate::comboard::imple::interface::Module_Config), super::interface::ModuleError> {
        Err(super::interface::ModuleError::new())
    }

    fn have_data_change(&self, current: &Box<dyn crate::modulestate::interface::ModuleValueParsable>, last: &Box<dyn crate::modulestate::interface::ModuleValueParsable>) -> bool {
        let current = current.as_any().downcast_ref::<SOILModuleData>().unwrap();
        let last = last.as_any().downcast_ref::<SOILModuleData>().unwrap();

        if difference_of(current.p0, last.p0, 2) {
            return true;
        } else if difference_of(current.p1, last.p1, 2) {
            return true;
        }else if difference_of(current.p2, last.p2, 2) {
            return true;
        }else if difference_of(current.p3, last.p3, 2) {
            return true;
        }else if difference_of(current.p4, last.p4, 2) {
            return true;
        }else if difference_of(current.p5, last.p5, 2) {
            return true;
        }else if difference_of(current.p6, last.p6, 2) {
            return true;
        }else if difference_of(current.p7, last.p7, 2) {
            return true;
        } else if difference_of(current.timestamp, last.timestamp, 60) {
            return true;
        }

        return false;
    }
}