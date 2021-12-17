use crate::protos::module::{WCModuleData, WCModuleConfig};
use super::relay::{configure_relay, get_outlet_data};
use protobuf::Message;

use core::any::Any;

pub struct AABValidator {}

impl super::interface::ModuleValue for WCModuleData {}

impl super::interface::ModuleValueParsable for WCModuleData {}

impl super::interface::ModuleValueValidator for AABValidator {

    fn convert_to_value(&self, value_event: &crate::comboard::imple::interface::ModuleValueValidationEvent) -> Result<Box<dyn super::interface::ModuleValueParsable>, super::interface::ModuleError> {
        let mut data = crate::protos::module::WCModuleData::new();
        data.p0 = get_outlet_data(value_event.buffer[0]);
        data.p1 = get_outlet_data(value_event.buffer[1]);
        data.p2 = get_outlet_data(value_event.buffer[2]);
        data.drain = get_outlet_data(value_event.buffer[3]);
        data.pump0 = get_outlet_data(value_event.buffer[4]);
        data.pump1 = get_outlet_data(value_event.buffer[5]);
        data.pump2 = get_outlet_data(value_event.buffer[6]);
        data.pump3 = get_outlet_data(value_event.buffer[7]);
        data.timestamp = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_secs() as i32;

        return Ok(Box::new(data));
    }

    fn apply_parse_config(&self, port: i32, _t: char, data: std::sync::Arc<Vec<u8>>,
        sender_comboard_config: & std::sync::mpsc::Sender<crate::comboard::imple::interface::Module_Config>,
        map_handler: & mut std::collections::HashMap<i32, tokio_util::sync::CancellationToken>
    ) -> Result<(Box<dyn protobuf::Message>, crate::comboard::imple::interface::Module_Config), super::interface::ModuleError> {

		
        let config: Box<WCModuleConfig> = Box::new(WCModuleConfig::parse_from_bytes(&data).map_err(|_e| super::interface::ModuleError::new())?);


        let mut buffer = [255; 8];

        configure_relay(config.has_p0(),0, &port, config.get_p0(), & mut buffer[0], sender_comboard_config, map_handler);
        configure_relay(config.has_p1(),1, &port, config.get_p1(), & mut buffer[1], sender_comboard_config, map_handler);
        configure_relay(config.has_p2(),2, &port, config.get_p2(), & mut buffer[2], sender_comboard_config, map_handler);
        configure_relay(config.has_drain(),3, &port, config.get_drain(), & mut buffer[3], sender_comboard_config, map_handler);
        configure_relay(config.has_pump0(),4, &port, config.get_pump0(), & mut buffer[4], sender_comboard_config, map_handler);
        configure_relay(config.has_pump1(),5, &port, config.get_pump1(), & mut buffer[5], sender_comboard_config, map_handler);
        configure_relay(config.has_pump2(),6, &port, config.get_pump2(), & mut buffer[6], sender_comboard_config, map_handler);
        configure_relay(config.has_pump3(),7, &port, config.get_pump3(), & mut buffer[7], sender_comboard_config, map_handler);

        return Ok((
            config,
            crate::comboard::imple::interface::Module_Config{
                port: port,
                buffer: buffer,
            },
        ));
    }

    fn have_data_change(&self, current: &Box<dyn crate::modulestate::interface::ModuleValueParsable>, last: &Box<dyn crate::modulestate::interface::ModuleValueParsable>) -> (bool, Vec<super::alarm::model::ValueChange<i32>>) {
        let current = current.as_any().downcast_ref::<WCModuleData>().unwrap();
        let last = last.as_any().downcast_ref::<WCModuleData>().unwrap();

        if current.p0.as_ref().unwrap().state != last.p0.as_ref().unwrap().state {
            return (true, vec![]);
        } else if current.p1.as_ref().unwrap().state != last.p1.as_ref().unwrap().state {
            return (true, vec![]);
        }  else if current.p2.as_ref().unwrap().state != last.p2.as_ref().unwrap().state {
            return (true, vec![]);
        } else if current.drain.as_ref().unwrap().state != last.drain.as_ref().unwrap().state {
            return (true, vec![]);
        } else if current.pump1.as_ref().unwrap().state != last.pump0.as_ref().unwrap().state {
            return (true, vec![]);
        } else if current.pump2.as_ref().unwrap().state != last.pump2.as_ref().unwrap().state {
            return (true, vec![]);
        } else if current.pump3.as_ref().unwrap().state != last.pump3.as_ref().unwrap().state {
            return (true, vec![]);
        } else if current.pump0.as_ref().unwrap().state != last.pump0.as_ref().unwrap().state {
            return (true, vec![]);
        }

        return (false, vec![]);
    }

}