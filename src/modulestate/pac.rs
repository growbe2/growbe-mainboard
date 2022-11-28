use protobuf::Message;

use crate::protos::module::PhoneAccelerationData;

use super::interface::ModuleError;

pub struct PACValidator {}

impl PACValidator {
    pub fn new() -> PACValidator {
        return PACValidator {};
    }
}

impl super::interface::ModuleValue for PhoneAccelerationData {}

impl super::interface::ModuleValueParsable for PhoneAccelerationData {}

impl super::interface::ModuleValueValidator for PACValidator {
    fn convert_to_value(
        &mut self,
        value_event: &crate::comboard::imple::interface::ModuleValueValidationEvent,
    ) -> Result<Box<dyn super::interface::ModuleValueParsable>, super::interface::ModuleError> {
        if let Ok(data) = PhoneAccelerationData::parse_from_bytes(&value_event.buffer) {
            return Ok(Box::new(data));
        }
        return Err(ModuleError::new());
    }

    fn apply_parse_config(
        &mut self,
        _port: i32,
        _t: &str,
        _data: std::sync::Arc<Vec<u8>>,
        _sender_comboard_config: &std::sync::mpsc::Sender<
            crate::comboard::imple::channel::ModuleConfig,
        >,
        _map_handler: &mut std::collections::HashMap<String, tokio_util::sync::CancellationToken>,
    ) -> Result<
        (
            Box<dyn protobuf::Message>,
            crate::comboard::imple::channel::ModuleConfig,
        ),
        super::interface::ModuleError,
    > {
        Err(super::interface::ModuleError::new())
    }

    fn remove_config(&mut self) -> Result<(), super::interface::ModuleError> {
        return Ok(());
    }

    fn have_data_change(
        &self,
        _current: &Box<dyn crate::modulestate::interface::ModuleValueParsable>,
        _last: &Box<dyn crate::modulestate::interface::ModuleValueParsable>,
    ) -> (bool, Vec<super::alarm::model::ValueChange<f32>>) {
        return (true, vec![]);
    }

    fn handle_command_validator(
        &mut self,
        _cmd: &str,
        _module_id: &String,
        _data: std::sync::Arc<Vec<u8>>,
        _sender_response: &std::sync::mpsc::Sender<crate::protos::message::ActionResponse>,
        _sender_socket: &std::sync::mpsc::Sender<(
            String,
            Box<dyn super::interface::ModuleValueParsable>,
        )>,
    ) -> Result<Option<Vec<super::interface::ModuleStateCmd>>, super::interface::ModuleError> {
        return Ok(None);
    }
}
