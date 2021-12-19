use crate::protos::module::{RelayModuleData, RelayModuleConfig, Actor};
use super::relay::{configure_relay, get_outlet_data};
use protobuf::Message;

pub struct AAPValidator {}

impl AAPValidator {
    fn new() -> AAPValidator {
        return AAPValidator {  };
    } 
}

impl super::interface::ModuleValue for RelayModuleData {}

impl super::interface::ModuleValueParsable for RelayModuleData {}

impl super::interface::ModuleValueValidator for AAPValidator {

    fn convert_to_value(&self, value_event: &crate::comboard::imple::interface::ModuleValueValidationEvent) -> Result<Box<dyn super::interface::ModuleValueParsable>, super::interface::ModuleError> {
        if value_event.buffer.len() >= 8 {
            let mut data = crate::protos::module::RelayModuleData::new();
            data.p0 = get_outlet_data(value_event.buffer[0]);
            data.p1 = get_outlet_data(value_event.buffer[1]);
            data.p2 = get_outlet_data(value_event.buffer[2]);
            data.p3 = get_outlet_data(value_event.buffer[3]);
            data.p4 = get_outlet_data(value_event.buffer[4]);
            data.p5 = get_outlet_data(value_event.buffer[5]);
            data.p6 = get_outlet_data(value_event.buffer[6]);
            data.p7 = get_outlet_data(value_event.buffer[7]);
            data.timestamp = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_secs() as i32;
            return Ok(Box::new(data));
        } else {
            return Err(super::interface::ModuleError::new());
        }
    }
    
    fn apply_parse_config(&mut self, port: i32, t: char, data: std::sync::Arc<Vec<u8>>,
        sender_comboard_config: & std::sync::mpsc::Sender<crate::comboard::imple::interface::Module_Config>,
        map_handler: & mut std::collections::HashMap<i32, tokio_util::sync::CancellationToken>
    ) -> Result<(Box<dyn protobuf::Message>, crate::comboard::imple::interface::Module_Config), super::interface::ModuleError> {

        let config: Box<RelayModuleConfig> = Box::new(RelayModuleConfig::parse_from_bytes(&data).map_err(|_e| super::interface::ModuleError::new())?);

        let mut buffer = [255; 8];
        /*let previous_owner: Option<Actor> = None;
        
        let new_owner_options = configure_relay(config.has_p0(),0, &port, config.get_p0(), & mut buffer[0], sender_comboard_config, map_handler, &previous_owner);
        configure_relay(config.has_p1(),1, &port, config.get_p1(), & mut buffer[1], sender_comboard_config, map_handler, &previous_owner);
        configure_relay(config.has_p2(),2, &port, config.get_p2(), & mut buffer[2], sender_comboard_config, map_handler, &previous_owner);
        configure_relay(config.has_p3(),3, &port, config.get_p3(), & mut buffer[3], sender_comboard_config, map_handler, &previous_owner);
        configure_relay(config.has_p4(),4, &port, config.get_p4(), & mut buffer[4], sender_comboard_config, map_handler, &previous_owner);
        configure_relay(config.has_p5(),5, &port, config.get_p5(), & mut buffer[5], sender_comboard_config, map_handler, &previous_owner);
        configure_relay(config.has_p6(),6, &port, config.get_p6(), & mut buffer[6], sender_comboard_config, map_handler, &previous_owner);
        configure_relay(config.has_p7(),7, &port, config.get_p7(), & mut buffer[7], sender_comboard_config, map_handler, &previous_owner);
    */
        return Ok((
            config,
            crate::comboard::imple::interface::Module_Config{
                port: port,
                buffer: buffer,
            },
        ));
    }

    fn have_data_change(&self, current: &Box<dyn crate::modulestate::interface::ModuleValueParsable>, last: &Box<dyn crate::modulestate::interface::ModuleValueParsable>) -> (bool, Vec<super::alarm::model::ValueChange<i32>>) {
        let current = current.as_any().downcast_ref::<RelayModuleData>().unwrap();
        let last = last.as_any().downcast_ref::<RelayModuleData>().unwrap();

        if current.p0.as_ref().unwrap().state != last.p0.as_ref().unwrap().state {
            return (true, vec![]);
        } else if current.p1.as_ref().unwrap().state != last.p1.as_ref().unwrap().state {
            return (true, vec![]);
        }  else if current.p2.as_ref().unwrap().state != last.p2.as_ref().unwrap().state {
            return (true, vec![]);
        } else if current.p3.as_ref().unwrap().state != last.p3.as_ref().unwrap().state {
            return (true, vec![]);
        } else if current.p4.as_ref().unwrap().state != last.p4.as_ref().unwrap().state {
            return (true, vec![]);
        } else if current.p5.as_ref().unwrap().state != last.p5.as_ref().unwrap().state {
            return (true, vec![]);
        } else if current.p6.as_ref().unwrap().state != last.p6.as_ref().unwrap().state {
            return (true, vec![]);
        } else if current.p7.as_ref().unwrap().state != last.p7.as_ref().unwrap().state {
            return (true, vec![]);
        }

        return (false, vec![]);
    }


}