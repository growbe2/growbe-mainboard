

pub trait ModuleValueValidator<T> {
    fn convertToValue(valueEvent: crate::comboard::imple::interface::ModuleValueValidationEvent) -> T;
}