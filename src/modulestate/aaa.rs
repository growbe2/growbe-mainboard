use crate::protos::module::{THLModuleData};
use crate::utils::validation::difference_of;

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
                data.timestamp = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_secs() as i32;
            } else {
                return Err(super::interface::ModuleError::new().message("could not parse value from buffer".to_string()))
            }
        }

        return Ok(Box::new(data));
    }
    
    fn apply_parse_config(&self, _port: i32, _t: char, _data: std::sync::Arc<Vec<u8>>, 
        _sender_comboard_config: & std::sync::mpsc::Sender<crate::comboard::imple::interface::Module_Config>,
        _map_handler: & mut std::collections::HashMap<i32, tokio_util::sync::CancellationToken>
    ) -> Result<(Box<dyn protobuf::Message>, crate::comboard::imple::interface::Module_Config), super::interface::ModuleError> {
        Err(super::interface::ModuleError::new())
    }

    fn have_data_change(&self, current: &Box<dyn crate::modulestate::interface::ModuleValueParsable>, last: &Box<dyn crate::modulestate::interface::ModuleValueParsable>) -> (bool, Vec<super::alarm::model::ValueChange<i32>>) {
        let current = current.as_any().downcast_ref::<THLModuleData>().unwrap();
        let last = last.as_any().downcast_ref::<THLModuleData>().unwrap();

        let mut vec = Vec::new();

        if difference_of(current.airTemperature, last.airTemperature, 0.5) {
            vec.push(super::alarm::model::ValueChange::<i32>{property: "airTemperature".to_string(), current_value: current.airTemperature as i32, previous_value: last.airTemperature as i32});
        }
        if difference_of(current.humidity, last.humidity, 0.5) {
            vec.push(super::alarm::model::ValueChange::<i32>{property: "humidity".to_string(), current_value: current.humidity as i32, previous_value: last.humidity as i32});
        }
        if difference_of(current.timestamp, last.timestamp, 30) {
            return (true, vec);
        }

        return (vec.len() > 0, vec);
    }


}