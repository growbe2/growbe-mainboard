use protobuf::Message;

use crate::protos::module::PhonePositionData;

use super::interface::ModuleError;


pub struct PPOValidator {}

impl PPOValidator {
    pub fn new() -> PPOValidator {
        return PPOValidator {
        };
    } 
}

impl super::interface::ModuleValue for PhonePositionData {}

impl super::interface::ModuleValueParsable for PhonePositionData {}

impl super::interface::ModuleValueValidator for PPOValidator {
    fn convert_to_value(&mut self, value_event: &crate::comboard::imple::interface::ModuleValueValidationEvent) -> Result<Box<dyn super::interface::ModuleValueParsable>, super::interface::ModuleError> {
        if let Ok(data )= PhonePositionData::parse_from_bytes(&value_event.buffer) {
            return Ok(Box::new(data));
        }
        return Err(ModuleError::new());
    }
    
    fn apply_parse_config(&mut self, _port: i32, _t: char, _data: std::sync::Arc<Vec<u8>>, 
        _sender_comboard_config: & std::sync::mpsc::Sender<crate::comboard::imple::interface::Module_Config>,
        _map_handler: & mut std::collections::HashMap<String, tokio_util::sync::CancellationToken>
    ) -> Result<(Box<dyn protobuf::Message>, crate::comboard::imple::interface::Module_Config), super::interface::ModuleError> {
        Err(super::interface::ModuleError::new())
    }

    fn remove_config(&mut self) -> Result<(), super::interface::ModuleError> {
        return Ok(());
    }

    fn have_data_change(&self, current: &Box<dyn crate::modulestate::interface::ModuleValueParsable>, last: &Box<dyn crate::modulestate::interface::ModuleValueParsable>) -> (bool, Vec<super::alarm::model::ValueChange<i32>>) {
        return (true, vec![]);
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