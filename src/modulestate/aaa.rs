use crate::protos::module::{THLModuleData};

use std::os::raw::{c_char, c_float};

extern "C" {
    pub fn strtof(s: *const u8, endp: *mut *mut c_char) -> c_float;
}

pub struct AAAValidator {}

impl super::interface::ModuleValue for THLModuleData {}

impl super::interface::ModuleValueParsable for THLModuleData {}

impl super::interface::ModuleValueValidator for AAAValidator {
    fn convert_to_value(&self, value_event: &crate::comboard::imple::interface::ModuleValueValidationEvent) -> Box<dyn super::interface::ModuleValueParsable> {
        let mut data = THLModuleData::new();

        let mut v = std::ptr::null_mut();

        unsafe {
            data.airTemperature = strtof(value_event.buffer.as_ptr(), &mut v);
            data.humidity = strtof(value_event.buffer.as_ptr().offset(100), &mut v);
        }

        return Box::new(data);
    }
    
    fn apply_parse_config(&self, port: i32, t: char, data: std::sync::Arc<Vec<u8>>, 
        sender_comboard_config: & std::sync::mpsc::Sender<crate::comboard::imple::interface::Module_Config>,
        map_handler: & mut std::collections::HashMap<i32, tokio::task::JoinHandle<()>>
    ) -> (Box<dyn protobuf::Message>, crate::comboard::imple::interface::Module_Config) {
        panic!("AAA has no config");
    }


}