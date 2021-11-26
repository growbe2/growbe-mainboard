use tokio::task::JoinHandle;
use core::any::Any;

pub trait ModuleValue {}
pub trait ModuleValueParsable: ModuleValue + protobuf::Message {}

impl ModuleValue for crate::protos::module::ModuleData {}
impl ModuleValueParsable for crate::protos::module::ModuleData {}


pub trait ModuleValueValidator {
    // need to be option result
    fn convert_to_value(&self, value_event: &crate::comboard::imple::interface::ModuleValueValidationEvent) -> Box<dyn ModuleValueParsable>;


    fn have_data_change(&self, current: &Box<dyn ModuleValueParsable>, last: &Box<dyn ModuleValueParsable>) -> bool;

    // need to be option result
    fn apply_parse_config(&self, port: i32, t: char, data: std::sync::Arc<Vec<u8>>,
        sender_comboard_config: & std::sync::mpsc::Sender<crate::comboard::imple::interface::Module_Config>,
        map_handler: & mut std::collections::HashMap<i32, tokio::task::JoinHandle<()>>,
    ) -> (Box<dyn protobuf::Message>, crate::comboard::imple::interface::Module_Config);
}