
pub trait ModuleValue {}

pub trait ModuleValueValidator<T: protobuf::Message + ModuleValue> {
    fn convert_to_value(&self, valueEvent: &crate::comboard::imple::interface::ModuleValueValidationEvent) -> T;
}