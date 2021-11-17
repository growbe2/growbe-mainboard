use crate::protos::module::{RelayModuleData, RelayModuleConfig, RelayOutletConfig, RelayOutletData, RelayOutletMode};
use protobuf::SingularPtrField;
use protobuf::Message;

pub struct AAPValidator {}

impl super::interface::ModuleValue for RelayModuleData {}

impl super::interface::ModuleValueParsable for RelayModuleData {}

fn get_outlet_data(value: u8) -> SingularPtrField<RelayOutletData> {
    let mut data = RelayOutletData::new();
    if value == 0 {
        data.set_state(false);
    } else if value == 1 {
        data.set_state(true);
    }
    return SingularPtrField::some(data);
}

fn configure_relay(
    has_field: bool,
    port: &i32,
    config: &RelayOutletConfig,
    buffer: & mut u8,
) -> () {
    if has_field {
        match config.mode {
            RelayOutletMode::MANUAL => {
                let manual_config = config.manual.as_ref().unwrap();
                if manual_config.state == true {
                    *buffer = 1;
                } else {
                    *buffer = 0;
                }

                if manual_config.duration > 0 {
                    println!("Duration my dude");
                }
            },
            RelayOutletMode::ALARM => {
                println!("Alarm my man");
            }
        }
    }
}

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
    fn apply_parse_config(&self, port: i32, t: char, data: std::sync::Arc<Vec<u8>>) -> (Box<dyn protobuf::Message>, crate::comboard::imple::interface::Module_Config) {

        let config: Box<RelayModuleConfig> = Box::new(RelayModuleConfig::parse_from_bytes(&data).unwrap());

        let mut buffer = [255; 8];

        configure_relay(config.has_p0(), &port, config.get_p0(), & mut buffer[0]);
        configure_relay(config.has_p1(), &port, config.get_p1(), & mut buffer[1]);
        configure_relay(config.has_p2(), &port, config.get_p2(), & mut buffer[2]);
        configure_relay(config.has_p3(), &port, config.get_p3(), & mut buffer[3]);
        configure_relay(config.has_p4(), &port, config.get_p4(), & mut buffer[4]);
        configure_relay(config.has_p5(), &port, config.get_p5(), & mut buffer[5]);
        configure_relay(config.has_p6(), &port, config.get_p6(), & mut buffer[6]);
        configure_relay(config.has_p7(), &port, config.get_p7(), & mut buffer[7]);

        return (
            config,
            crate::comboard::imple::interface::Module_Config{
                port: port,
                buffer: buffer,
            }
        );
    }



}