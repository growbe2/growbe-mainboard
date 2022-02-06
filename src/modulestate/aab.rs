use crate::protos::module::{WCModuleData, WCModuleConfig, Actor};
use super::{relay::{get_outlet_data, configure::configure_relay, physical_relay::ActionPortUnion}};
use super::actor::{get_owner};
use std::collections::HashMap;
use protobuf::Message;
use crate::modulestate::relay::BatchRelay;

pub struct AABValidator {
    pub actors_property: HashMap<String, Actor>,
}

impl AABValidator {
    pub fn new() -> AABValidator {
        return AABValidator {
            actors_property: HashMap::new(),
        };
    }
}

impl super::interface::ModuleValue for WCModuleData {}

impl super::interface::ModuleValueParsable for WCModuleData {}

impl super::interface::ModuleValueValidator for AABValidator {

    fn convert_to_value(&mut self, value_event: &crate::comboard::imple::interface::ModuleValueValidationEvent) -> Result<Box<dyn super::interface::ModuleValueParsable>, super::interface::ModuleError> {
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

    fn remove_config(&mut self) -> Result<(), super::interface::ModuleError> {

        return Ok(());
    }

    fn apply_parse_config(&mut self, port: i32, _t: char, data: std::sync::Arc<Vec<u8>>,
        sender_comboard_config: & std::sync::mpsc::Sender<crate::comboard::imple::interface::Module_Config>,
        map_handler: & mut std::collections::HashMap<String, tokio_util::sync::CancellationToken>
    ) -> Result<(Box<dyn protobuf::Message>, crate::comboard::imple::interface::Module_Config), super::interface::ModuleError> {

		
        let config: Box<WCModuleConfig> = Box::new(WCModuleConfig::parse_from_bytes(&data).map_err(|_e| super::interface::ModuleError::new())?);

        let buffer = [255; 8];

        let previous_owner: Option<&Actor> = get_owner(&self.actors_property, "p0");
        
        let mut batch_relay = super::relay::physical_relay::BatchPhysicalRelay{
            action_port: ActionPortUnion::new_port(0),
            buffer: [255; 8],
            auto_send: false,
            port: port,
            sender: sender_comboard_config.clone(),
        };
        configure_relay(config.has_p0(), config.get_p0(), &mut batch_relay, map_handler, previous_owner);

        batch_relay.action_port.port = 1;
        configure_relay(config.has_p1(), config.get_p1(), &mut batch_relay, map_handler, previous_owner);

        batch_relay.action_port.port = 2;
        configure_relay(config.has_p2(), config.get_p2(), &mut batch_relay, map_handler, previous_owner);

        batch_relay.action_port.port = 3;
        configure_relay(config.has_drain(), config.get_drain(), &mut batch_relay, map_handler, previous_owner);

        batch_relay.action_port.port = 4;
        configure_relay(config.has_pump0(), config.get_pump0(), &mut batch_relay, map_handler, previous_owner);

        batch_relay.action_port.port = 5;
        configure_relay(config.has_pump1(), config.get_pump1(), &mut batch_relay, map_handler, previous_owner);

        batch_relay.action_port.port = 6;
        configure_relay(config.has_pump2(), config.get_pump2(), &mut batch_relay, map_handler, previous_owner);

        batch_relay.action_port.port = 7;
        configure_relay(config.has_pump3(), config.get_pump3(), &mut batch_relay, map_handler, previous_owner);

        batch_relay.execute().unwrap();

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

    fn handle_command_validator(
        &mut self,
        _cmd: &str,
        _module_id: &String,
        _data: std::sync::Arc<Vec<u8>>,
        _sender_response: &std::sync::mpsc::Sender<crate::protos::message::ActionResponse>,
        _sender_socket: & std::sync::mpsc::Sender<(String, Box<dyn super::interface::ModuleValueParsable>)>,
    ) -> Result<Option<Vec<super::interface::ModuleStateCmd>>, super::interface::ModuleError> {
        return Ok(None);
    }

}