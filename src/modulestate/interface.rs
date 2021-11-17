
pub trait ModuleValue {}
pub trait ModuleValueParsable: ModuleValue + protobuf::Message {}

impl ModuleValue for crate::protos::module::ModuleData {}
impl ModuleValueParsable for crate::protos::module::ModuleData {}


pub trait ModuleValueValidator {
    fn convert_to_value(&self, value_event: &crate::comboard::imple::interface::ModuleValueValidationEvent) -> Box<dyn ModuleValueParsable>;

    fn apply_parse_config(&self, port: i32, t: char, data: std::sync::Arc<Vec<u8>>) -> (Box<dyn protobuf::Message>, crate::comboard::imple::interface::Module_Config);
}