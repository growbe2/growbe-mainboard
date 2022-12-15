use protobuf::Message;

use crate::protos::module::ComputerStatsData;

use crate::modulestate::interface::ModuleError;

pub struct CSSValidator {}

impl CSSValidator {
    pub fn new() -> CSSValidator {
        return CSSValidator {};
    }
}

impl crate::modulestate::interface::ModuleValue for ComputerStatsData {}

impl crate::modulestate::interface::ModuleValueParsable for ComputerStatsData {}

impl crate::modulestate::interface::ModuleValueValidator for CSSValidator {
    fn convert_to_value(
        &mut self,
        value_event: &crate::comboard::imple::interface::ModuleValueValidationEvent,
    ) -> Result<
        Box<dyn crate::modulestate::interface::ModuleValueParsable>,
        crate::modulestate::interface::ModuleError,
    > {
        match ComputerStatsData::parse_from_bytes(&value_event.buffer) {
            Ok(data) => Ok(Box::new(data)),
            Err(err) => Err(ModuleError::new().message(err.to_string())),
        }
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
        crate::modulestate::interface::ModuleError,
    > {
        return Err(ModuleError::new());
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
        _sender_socket: &tokio::sync::mpsc::Sender<(
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
