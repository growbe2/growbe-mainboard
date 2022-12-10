use protobuf::Message;

use crate::protos::module::{PhoneStreamingConfig, PhoneStreamingData};

use crate::modulestate::interface::ModuleError;

pub struct PCSValidator {}

impl PCSValidator {
    pub fn new() -> PCSValidator {
        return PCSValidator {};
    }
}

impl crate::modulestate::interface::ModuleValue for PhoneStreamingData {}

impl crate::modulestate::interface::ModuleValueParsable for PhoneStreamingData {}

impl crate::modulestate::interface::ModuleValueValidator for PCSValidator {
    fn convert_to_value(
        &mut self,
        value_event: &crate::comboard::imple::interface::ModuleValueValidationEvent,
    ) -> Result<
        Box<dyn crate::modulestate::interface::ModuleValueParsable>,
        crate::modulestate::interface::ModuleError,
    > {
        match PhoneStreamingData::parse_from_bytes(&value_event.buffer) {
            Ok(data) => Ok(Box::new(data)),
            Err(err) => Err(ModuleError::new().message(err.to_string())),
        }
    }

    fn apply_parse_config(
        &mut self,
        port: i32,
        _t: &str,
        data: std::sync::Arc<Vec<u8>>,
        _sender_comboard_config: &std::sync::mpsc::Sender<
            crate::comboard::imple::channel::ModuleConfig,
        >,
        _map_handler: &mut std::collections::HashMap<String, tokio_util::sync::CancellationToken>,
    ) -> Result<
        (
            Box<dyn protobuf::Message>,
            crate::comboard::imple::channel::ModuleConfig,
        ),
        crate::modulestate::interface::ModuleError,
    > {
        let config: Box<PhoneStreamingConfig> =
            Box::new(PhoneStreamingConfig::parse_from_bytes(&data).map_err(|e| {
                crate::modulestate::interface::ModuleError::new().message(e.to_string())
            })?);

        return Ok((
            config,
            crate::comboard::imple::channel::ModuleConfig {
                port: port,
                data: data.to_vec(),
            },
        ));
    }

    fn remove_config(&mut self) -> Result<(), crate::modulestate::interface::ModuleError> {
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
        _sender_response: &std::sync::mpsc::Sender<crate::protos::message::ActionResponse>,
        _sender_socket: &std::sync::mpsc::Sender<(
            String,
            Box<dyn crate::modulestate::interface::ModuleValueParsable>,
        )>,
    ) -> Result<
        Option<Vec<crate::modulestate::interface::ModuleStateCmd>>,
        crate::modulestate::interface::ModuleError,
    > {
        return Ok(None);
    }
}