use crate::protos::module::{RelayModuleData, RelayModuleConfig};
use super::relay::{configure_relay, get_outlet_data};
use protobuf::Message;

pub struct AAPValidator {}

impl super::interface::ModuleValue for RelayModuleData {}

impl super::interface::ModuleValueParsable for RelayModuleData {}

impl super::interface::ModuleValueValidator for AAPValidator {

    fn convert_to_value(&self, value_event: &crate::comboard::imple::interface::ModuleValueValidationEvent) -> Box<dyn super::interface::ModuleValueParsable> {
        let mut data = crate::protos::module::RelayModuleData::new();
        data.p0 = get_outlet_data(value_event.buffer[0]);
        data.p1 = get_outlet_data(value_event.buffer[1]);
        data.p2 = get_outlet_data(value_event.buffer[2]);
        data.p3 = get_outlet_data(value_event.buffer[3]);
        data.p4 = get_outlet_data(value_event.buffer[4]);
        data.p5 = get_outlet_data(value_event.buffer[5]);
        data.p6 = get_outlet_data(value_event.buffer[6]);
        data.p7 = get_outlet_data(value_event.buffer[7]);
        return Box::new(data);
    }
    fn apply_parse_config(&self, port: i32, t: char, data: std::sync::Arc<Vec<u8>>,
        sender_comboard_config: & std::sync::mpsc::Sender<crate::comboard::imple::interface::Module_Config>,
        map_handler: & mut std::collections::HashMap<i32, tokio::task::JoinHandle<()>>
    ) -> (Box<dyn protobuf::Message>, crate::comboard::imple::interface::Module_Config) {

        let config: Box<RelayModuleConfig> = Box::new(RelayModuleConfig::parse_from_bytes(&data).unwrap());


        let mut buffer = [255; 8];

        configure_relay(config.has_p0(),0, &port, config.get_p0(), & mut buffer[0], sender_comboard_config, map_handler);
        configure_relay(config.has_p1(),1, &port, config.get_p1(), & mut buffer[1], sender_comboard_config, map_handler);
        configure_relay(config.has_p2(),2, &port, config.get_p2(), & mut buffer[2], sender_comboard_config, map_handler);
        configure_relay(config.has_p3(),3, &port, config.get_p3(), & mut buffer[3], sender_comboard_config, map_handler);
        configure_relay(config.has_p4(),4, &port, config.get_p4(), & mut buffer[4], sender_comboard_config, map_handler);
        configure_relay(config.has_p5(),5, &port, config.get_p5(), & mut buffer[5], sender_comboard_config, map_handler);
        configure_relay(config.has_p6(),6, &port, config.get_p6(), & mut buffer[6], sender_comboard_config, map_handler);
        configure_relay(config.has_p7(),7, &port, config.get_p7(), & mut buffer[7], sender_comboard_config, map_handler);

        return (
            config,
            crate::comboard::imple::interface::Module_Config{
                port: port,
                buffer: buffer,
            },
        );
    }



}