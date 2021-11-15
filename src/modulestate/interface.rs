
pub trait ModuleValue {}
pub trait ModuleValueParsable: ModuleValue + protobuf::Message {}

impl ModuleValue for crate::protos::module::ModuleData {}
impl ModuleValueParsable for crate::protos::module::ModuleData {}

pub trait ModuleValueValidator {
    fn convert_to_value(&self, value_event: &crate::comboard::imple::interface::ModuleValueValidationEvent) -> Box<dyn ModuleValueParsable>;
}