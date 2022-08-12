use crate::protos::module::{RelayModuleData, RelayModuleConfig, Actor};
use super::relay::configure::configure_relay;
use super::relay::physical_relay::ActionPortUnion;
use super::relay::{get_outlet_data};
use super::actor::{get_owner};
use protobuf::Message;
use crate::modulestate::relay::BatchRelay;

pub struct AAPValidator {
    pub actors_property: std::collections::HashMap<String, Actor>,
    pub previous_config: RelayModuleConfig,
}

impl AAPValidator {
    pub fn new() -> AAPValidator {
        return AAPValidator {
            actors_property: std::collections::HashMap::new(),
            previous_config: RelayModuleConfig::new(),
        };
    } 
}

impl super::interface::ModuleValue for RelayModuleData {}

impl super::interface::ModuleValueParsable for RelayModuleData {}

impl super::interface::ModuleValueValidator for AAPValidator {

    fn convert_to_value(&mut self, value_event: &crate::comboard::imple::interface::ModuleValueValidationEvent) -> Result<Box<dyn super::interface::ModuleValueParsable>, super::interface::ModuleError> {
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

    fn remove_config(&mut self) -> Result<(), super::interface::ModuleError> {

        return Ok(());
    }
    
    fn apply_parse_config(&mut self, port: i32, _t: char, data: std::sync::Arc<Vec<u8>>,
        sender_comboard_config: & std::sync::mpsc::Sender<crate::comboard::imple::interface::Module_Config>,
        map_handler: & mut std::collections::HashMap<String, tokio_util::sync::CancellationToken>
    ) -> Result<(Box<dyn protobuf::Message>, crate::comboard::imple::interface::Module_Config), super::interface::ModuleError> {

        let config: Box<RelayModuleConfig> = Box::new(RelayModuleConfig::parse_from_bytes(&data).map_err(|_e| super::interface::ModuleError::new())?);

        let buffer = [255; 8];
        let previous_owner: Option<&Actor> = get_owner(&self.actors_property, "p0");
        let mut batch_relay = super::relay::physical_relay::BatchPhysicalRelay{
            action_port: ActionPortUnion::new_port(0),
            buffer: [255; 8],
            port: port,
            auto_send: false,
            sender: sender_comboard_config.clone(),
        };
        configure_relay(config.has_p0(), config.get_p0(),self.previous_config.has_p0(), self.previous_config.get_p0(), &mut batch_relay, map_handler, previous_owner);

        batch_relay.action_port.port = 1;
        configure_relay(config.has_p1(), config.get_p1(),self.previous_config.has_p1(), self.previous_config.get_p1(), &mut batch_relay, map_handler, previous_owner);

        batch_relay.action_port.port = 2;
        configure_relay(config.has_p2(), config.get_p2(),self.previous_config.has_p2(), self.previous_config.get_p2(), &mut batch_relay, map_handler, previous_owner);

        batch_relay.action_port.port = 3;
        configure_relay(config.has_p3(), config.get_p3(),self.previous_config.has_p3(), self.previous_config.get_p3(), &mut batch_relay, map_handler, previous_owner);

        batch_relay.action_port.port = 4;
        configure_relay(config.has_p4(), config.get_p4(),self.previous_config.has_p4(), self.previous_config.get_p4(), &mut batch_relay, map_handler, previous_owner);

        batch_relay.action_port.port = 5;
        configure_relay(config.has_p5(), config.get_p5(),self.previous_config.has_p5(), self.previous_config.get_p5(), &mut batch_relay, map_handler, previous_owner);

        batch_relay.action_port.port = 6;
        configure_relay(config.has_p6(), config.get_p6(), self.previous_config.has_p6(), self.previous_config.get_p6(),&mut batch_relay, map_handler, previous_owner);

        batch_relay.action_port.port = 7;
        configure_relay(config.has_p7(), config.get_p7(), self.previous_config.has_p7(), self.previous_config.get_p7(), &mut batch_relay, map_handler, previous_owner);

        batch_relay.execute().unwrap();

        //self.previous_config = *config.clone();

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