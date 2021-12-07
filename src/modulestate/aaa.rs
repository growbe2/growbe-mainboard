use crate::protos::module::{THLModuleData};

use std::os::raw::{c_char, c_float};

extern "C" {
    pub fn strtof(s: *const u8, endp: *mut *mut c_char) -> c_float;
}

pub struct AAAValidator {}

impl super::interface::ModuleValue for THLModuleData {}

impl super::interface::ModuleValueParsable for THLModuleData {}

impl super::interface::ModuleValueValidator for AAAValidator {
    fn convert_to_value(&self, value_event: &crate::comboard::imple::interface::ModuleValueValidationEvent) -> Result<Box<dyn super::interface::ModuleValueParsable>, super::interface::ModuleError> {
        let mut data = THLModuleData::new();

        let mut v = std::ptr::null_mut();

        unsafe {
            if value_event.buffer.len() > 150 {
                data.airTemperature = strtof(value_event.buffer.as_ptr(), &mut v);
                data.humidity = strtof(value_event.buffer.as_ptr().offset(100), &mut v);
            } else {
                return Err(super::interface::ModuleError::new().message("could not parse value from buffer".to_string()))
            }
        }

        return Ok(Box::new(data));
    }
    
    fn apply_parse_config(&self, _port: i32, _t: char, _data: std::sync::Arc<Vec<u8>>, 
        _sender_comboard_config: & std::sync::mpsc::Sender<crate::comboard::imple::interface::Module_Config>,
        _map_handler: & mut std::collections::HashMap<i32, tokio::task::JoinHandle<()>>
    ) -> Result<(Box<dyn protobuf::Message>, crate::comboard::imple::interface::Module_Config), super::interface::ModuleError> {
        Err(super::interface::ModuleError::new())
    }

    fn have_data_change(&self, _current: &Box<dyn crate::modulestate::interface::ModuleValueParsable>, _last: &Box<dyn crate::modulestate::interface::ModuleValueParsable>) -> bool {
        return true;
    }


}