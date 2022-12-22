use protobuf::Message;

use crate::protos::module::PhonePressureData;

use crate::modulestate::interface::ModuleError;

pub struct PPRValidator {}

impl PPRValidator {
    pub fn new() -> PPRValidator {
        return PPRValidator {};
    }
}

impl crate::modulestate::interface::ModuleValue for PhonePressureData {}

impl crate::modulestate::interface::ModuleValueParsable for PhonePressureData {}

impl crate::modulestate::interface::ModuleValueValidator for PPRValidator {
    fn convert_to_value(
        &mut self,
        value_event: &crate::comboard::imple::interface::ModuleValueValidationEvent,
    ) -> Result<
        Box<dyn crate::modulestate::interface::ModuleValueParsable>,
        crate::modulestate::interface::ModuleError,
    > {
        match PhonePressureData::parse_from_bytes(&value_event.buffer) {
            Ok(data) => Ok(Box::new(data)),
            Err(err) => Err(ModuleError::new().message(err.to_string())),
        }
    }

    fn apply_parse_config(
        &mut self,
        _port: i32,
        _t: &str,
        _data: std::sync::Arc<Vec<u8>>,
        _sender_comboard_config: &tokio::sync::mpsc::Sender<
            crate::comboard::imple::channel::ModuleConfig,
        >,
        _map_handler: &mut std::collections::HashMap<String, tokio_util::sync::CancellationToken>,
        actor: crate::protos::module::Actor,
    ) -> Result<
        (
            Box<dyn protobuf::Message>,
            crate::comboard::imple::channel::ModuleConfig,
        ),
        crate::modulestate::interface::ModuleError,
    > {
        Err(crate::modulestate::interface::ModuleError::new())
    }

    fn remove_config(&mut self,_actor: crate::protos::module::Actor,) -> Result<(), crate::modulestate::interface::ModuleError> {
        return Ok(());
    }

    fn have_data_change(
        &self,
        _current: &Box<dyn crate::modulestate::interface::ModuleValueParsable>,
        _last: &Box<dyn crate::modulestate::interface::ModuleValueParsable>,
    ) -> (
        bool,
        Vec<crate::modulestate::alarm::model::ValueChange<f32>>,
    ) {
        return (true, vec![]);
    }

    fn handle_command_validator(
        &mut self,
        _cmd: &str,
        _module_id: &String,
        _data: std::sync::Arc<Vec<u8>>,
        _sender_response: tokio::sync::oneshot::Sender<crate::protos::message::ActionResponse>,
        _sender_socket: &tokio::sync::mpsc::Sender<(
            String,
            Box<dyn crate::modulestate::interface::ModuleValueParsable>,
        )>,
_actor: crate::protos::module::Actor,
    ) -> Result<
        Option<Vec<crate::modulestate::interface::ModuleStateCmd>>,
        crate::modulestate::interface::ModuleError,
    > {
        return Ok(None);
    }
}
