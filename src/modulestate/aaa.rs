use crate::protos::module::{THLModuleData};

pub struct AAAValidator {}

impl super::interface::ModuleValue for THLModuleData {}

impl super::interface::ModuleValueParsable for THLModuleData {}

impl super::interface::ModuleValueValidator for AAAValidator {
    fn convert_to_value(&self, value_event: &crate::comboard::imple::interface::ModuleValueValidationEvent) -> Box<dyn super::interface::ModuleValueParsable> {
        let mut data = THLModuleData::new();
        data.airTemperature = value_event.buffer[0] as f32;
        data.humidity = value_event.buffer[1] as f32;
        return Box::new(data);
    }
    
    fn apply_parse_config(&self, port: i32, t: char, data: std::sync::Arc<Vec<u8>>, 
        sender_comboard_config: & std::sync::mpsc::Sender<crate::comboard::imple::interface::Module_Config>,
        map_handler: & mut std::collections::HashMap<i32, tokio::task::JoinHandle<()>>
    ) -> (Box<dyn protobuf::Message>, crate::comboard::imple::interface::Module_Config) {
        panic!("AAA has no config");
    }


}