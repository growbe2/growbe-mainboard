use crate::protos::module::{THLModuleData};
use crate::utils::validation::{difference_of, round_decimal};

use std::os::raw::{c_char, c_float};

extern "C" {
    pub fn strtof(s: *const u8, endp: *mut *mut c_char) -> c_float;
}

pub struct AAAValidator {}

impl AAAValidator {
    pub fn new() -> AAAValidator {
        return AAAValidator {
        };
    } 
}

impl super::interface::ModuleValue for THLModuleData {}

impl super::interface::ModuleValueParsable for THLModuleData {}

impl super::interface::ModuleValueValidator for AAAValidator {
    fn convert_to_value(&mut self, value_event: &crate::comboard::imple::interface::ModuleValueValidationEvent) -> Result<Box<dyn super::interface::ModuleValueParsable>, super::interface::ModuleError> {
        let mut data = THLModuleData::new();

        let mut v = std::ptr::null_mut();

        unsafe {
            if value_event.buffer.len() > 150 {
                data.airTemperature = round_decimal(strtof(value_event.buffer.as_ptr(), &mut v));
                data.humidity = round_decimal(strtof(value_event.buffer.as_ptr().offset(100), &mut v));
                data.timestamp = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_secs() as i32;
            } else {
                return Err(super::interface::ModuleError::new().message("could not parse value from buffer".to_string()))
            }
        }

        return Ok(Box::new(data));
    }
    
    fn apply_parse_config(&mut self, _port: i32, _t: &str, _data: std::sync::Arc<Vec<u8>>, 
        _sender_comboard_config: & std::sync::mpsc::Sender<crate::comboard::imple::channel::ModuleConfig>,
        _map_handler: & mut std::collections::HashMap<String, tokio_util::sync::CancellationToken>
    ) -> Result<(Box<dyn protobuf::Message>, crate::comboard::imple::channel::ModuleConfig), super::interface::ModuleError> {
        Err(super::interface::ModuleError::new())
    }

    fn remove_config(&mut self) -> Result<(), super::interface::ModuleError> {
        return Ok(());
    }

    fn have_data_change(&self, current: &Box<dyn crate::modulestate::interface::ModuleValueParsable>, last: &Box<dyn crate::modulestate::interface::ModuleValueParsable>) -> (bool, Vec<super::alarm::model::ValueChange<f32>>) {
        let current = current.as_any().downcast_ref::<THLModuleData>().unwrap();
        let last = last.as_any().downcast_ref::<THLModuleData>().unwrap();

        let mut vec = Vec::new();

        if difference_of(current.airTemperature, last.airTemperature, 0.1) {
            vec.push(super::alarm::model::ValueChange::<f32>{property: "airTemperature".to_string(), current_value: current.airTemperature, previous_value: last.airTemperature});
        }
        if difference_of(current.humidity, last.humidity, 0.5) {
            vec.push(super::alarm::model::ValueChange::<f32>{property: "humidity".to_string(), current_value: current.humidity, previous_value: last.humidity});
        }
        if difference_of(current.timestamp, last.timestamp, 30) {
            return (true, vec);
        }

        return (vec.len() > 0, vec);
    }

    fn handle_command_validator(
        &mut self,
        _cmd: &str,
        _module_id: &String,
        _data: std::sync::Arc<Vec<u8>>,
        _sender_response: &std::sync::mpsc::Sender<crate::protos::message::ActionResponse>,
        _sender_socket: & std::sync::mpsc::Sender<(String, Box<dyn super::interface::ModuleValueParsable>)>,
    ) -> Result<Option<Vec<super::interface::ModuleStateCmd>>, super::interface::ModuleError> {
        return Ok(None);
    }
}
