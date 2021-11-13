
use crate::protos::module::{SOILModuleData};

pub struct AAP {}

impl super::interface::ModuleValueValidator<SOILModuleData> for AAP {

    fn convertToValue(_valueEvent: crate::comboard::imple::interface::ModuleValueValidationEvent) -> SOILModuleData {
        return SOILModuleData::new();
    }

}