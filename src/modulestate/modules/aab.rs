use crate::modulestate::actor::get_owner;
use crate::modulestate::relay::BatchRelay;
use crate::modulestate::relay::{
    configure::configure_relay, get_outlet_data, physical_relay::ActionPortUnion,
};
use crate::protos::module::{Actor, RelayOutletConfig, WCModuleConfig, WCModuleData};
use crate::set_property;
use protobuf::Message;
use protobuf::SingularPtrField;
use std::collections::HashMap;

pub struct AABValidator {
    pub actors_property: HashMap<String, Actor>,
    pub previous_config: WCModuleConfig,
}

impl AABValidator {
    pub fn new() -> AABValidator {
        return AABValidator {
            actors_property: HashMap::new(),
            previous_config: WCModuleConfig::new(),
        };
    }
}

impl crate::modulestate::interface::ModuleValue for WCModuleData {}

impl crate::modulestate::interface::ModuleValueParsable for WCModuleData {}

impl crate::modulestate::interface::ModuleValueValidator for AABValidator {
    fn convert_to_value(
        &mut self,
        value_event: &crate::comboard::imple::interface::ModuleValueValidationEvent,
    ) -> Result<
        Box<dyn crate::modulestate::interface::ModuleValueParsable>,
        crate::modulestate::interface::ModuleError,
    > {
        let mut data = crate::protos::module::WCModuleData::new();
        data.p0 = get_outlet_data(value_event.buffer[0]);
        data.p1 = get_outlet_data(value_event.buffer[1]);
        data.p2 = get_outlet_data(value_event.buffer[2]);
        data.drain = get_outlet_data(value_event.buffer[3]);
        data.pump0 = get_outlet_data(value_event.buffer[4]);
        data.pump1 = get_outlet_data(value_event.buffer[5]);
        data.pump2 = get_outlet_data(value_event.buffer[6]);
        data.pump3 = get_outlet_data(value_event.buffer[7]);
        data.timestamp = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i32;

        return Ok(Box::new(data));
    }

    fn remove_config(&mut self) -> Result<(), crate::modulestate::interface::ModuleError> {
        return Ok(());
    }

    fn apply_parse_config(
        &mut self,
        port: i32,
        t: &str,
        data: std::sync::Arc<Vec<u8>>,
        sender_comboard_config: &std::sync::mpsc::Sender<
            crate::comboard::imple::channel::ModuleConfig,
        >,
        map_handler: &mut std::collections::HashMap<String, tokio_util::sync::CancellationToken>,
    ) -> Result<
        (
            Box<dyn protobuf::Message>,
            crate::comboard::imple::channel::ModuleConfig,
        ),
        crate::modulestate::interface::ModuleError,
    > {
        let config: Box<WCModuleConfig> = if t == "AAB" {
            Box::new(
                WCModuleConfig::parse_from_bytes(&data)
                    .map_err(|_e| crate::modulestate::interface::ModuleError::new())?,
            )
        } else {
            let property = t.split(":").last();
            if property.is_none() {
                Box::new(WCModuleConfig::new())
            } else {
                let property = property.unwrap();
                let relay_outlet = RelayOutletConfig::parse_from_bytes(&data)
                    .map_err(|_e| crate::modulestate::interface::ModuleError::new())?;

                let mut config = self.previous_config.clone();
                set_property!(
                    config,
                    property,
                    relay_outlet,
                    p0,
                    p1,
                    p2,
                    drain,
                    pump0,
                    pump1,
                    pump2,
                    pump3
                );

                Box::new(config)
            }
        };

        let buffer = [255; 8];

        let previous_owner: Option<&Actor> = get_owner(&self.actors_property, "p0");

        let mut batch_relay = crate::modulestate::relay::physical_relay::BatchPhysicalRelay {
            action_port: ActionPortUnion::new_port(0),
            buffer: [255; 8],
            auto_send: false,
            port: port,
            sender: sender_comboard_config.clone(),
        };

        configure_relay(
            config.has_p0(),
            config.get_p0(),
            self.previous_config.has_p0(),
            self.previous_config.get_p0(),
            &mut batch_relay,
            map_handler,
            previous_owner,
        );

        batch_relay.action_port.port = 1;
        configure_relay(
            config.has_p1(),
            config.get_p1(),
            self.previous_config.has_p1(),
            self.previous_config.get_p1(),
            &mut batch_relay,
            map_handler,
            previous_owner,
        );

        batch_relay.action_port.port = 2;
        configure_relay(
            config.has_p2(),
            config.get_p2(),
            self.previous_config.has_p2(),
            self.previous_config.get_p2(),
            &mut batch_relay,
            map_handler,
            previous_owner,
        );

        batch_relay.action_port.port = 3;
        configure_relay(
            config.has_drain(),
            config.get_drain(),
            self.previous_config.has_drain(),
            self.previous_config.get_drain(),
            &mut batch_relay,
            map_handler,
            previous_owner,
        );

        batch_relay.action_port.port = 4;
        configure_relay(
            config.has_pump0(),
            config.get_pump0(),
            self.previous_config.has_pump0(),
            self.previous_config.get_pump0(),
            &mut batch_relay,
            map_handler,
            previous_owner,
        );

        batch_relay.action_port.port = 5;
        configure_relay(
            config.has_pump1(),
            config.get_pump1(),
            self.previous_config.has_pump1(),
            self.previous_config.get_pump1(),
            &mut batch_relay,
            map_handler,
            previous_owner,
        );

        batch_relay.action_port.port = 6;
        configure_relay(
            config.has_pump2(),
            config.get_pump2(),
            self.previous_config.has_pump2(),
            self.previous_config.get_pump2(),
            &mut batch_relay,
            map_handler,
            previous_owner,
        );

        batch_relay.action_port.port = 7;
        configure_relay(
            config.has_pump3(),
            config.get_pump3(),
            self.previous_config.has_pump3(),
            self.previous_config.get_pump3(),
            &mut batch_relay,
            map_handler,
            previous_owner,
        );

        batch_relay.execute().unwrap();

        self.previous_config = *config.clone();

        return Ok((
            config,
            crate::comboard::imple::channel::ModuleConfig {
                port,
                data: buffer.try_into().unwrap(),
            },
        ));
    }

    fn have_data_change(
        &self,
        current: &Box<dyn crate::modulestate::interface::ModuleValueParsable>,
        last: &Box<dyn crate::modulestate::interface::ModuleValueParsable>,
    ) -> (
        bool,
        Vec<crate::modulestate::alarm::model::ValueChange<f32>>,
    ) {
        let current = current.as_any().downcast_ref::<WCModuleData>().unwrap();
        let last = last.as_any().downcast_ref::<WCModuleData>().unwrap();

        if current.p0.as_ref().unwrap().state != last.p0.as_ref().unwrap().state {
            return (true, vec![]);
        } else if current.p1.as_ref().unwrap().state != last.p1.as_ref().unwrap().state {
            return (true, vec![]);
        } else if current.p2.as_ref().unwrap().state != last.p2.as_ref().unwrap().state {
            return (true, vec![]);
        } else if current.drain.as_ref().unwrap().state != last.drain.as_ref().unwrap().state {
            return (true, vec![]);
        } else if current.pump1.as_ref().unwrap().state != last.pump1.as_ref().unwrap().state {
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
        _sender_socket: &tokio::sync::mpsc::Sender<(
            String,
            Box<dyn crate::modulestate::interface::ModuleValueParsable>,
        )>,
    ) -> Result<
        Option<Vec<crate::modulestate::interface::ModuleStateCmd>>,
        crate::modulestate::interface::ModuleError,
    > {
        return Ok(None);
    }
}

#[cfg(test)]
mod tests {

    use std::{
        collections::HashMap,
        sync::{mpsc::channel, Arc},
        time::Duration,
    };

    use tokio_util::sync::CancellationToken;

    use crate::{
        comboard::imple::channel::ModuleConfig, modulestate::interface::ModuleValueValidator,
        protos::module::ManualConfig,
    };

    use super::*;

    #[test]
    fn module_aab_apply_full_config() {
        let mut validator = AABValidator::new();
        let (s, _r) = channel::<ModuleConfig>();
        let config = WCModuleConfig::new();
        let mut map_handler: HashMap<String, CancellationToken> = HashMap::new();

        validator
            .apply_parse_config(
                0,
                "AAB",
                Arc::new(config.write_to_bytes().unwrap()),
                &s,
                &mut map_handler,
            )
            .unwrap();
    }

    #[test]
    fn module_aab_apply_partial_config() {
        let mut validator = AABValidator::new();
        let (s, r) = channel::<ModuleConfig>();
        let mut config = RelayOutletConfig::new();
        let manual = ManualConfig {
            state: true,
            ..Default::default()
        };
        config.set_manual(manual);
        let mut map_handler: HashMap<String, CancellationToken> = HashMap::new();

        let (c, _) = validator
            .apply_parse_config(
                0,
                "AAB:p0",
                Arc::new(config.write_to_bytes().unwrap()),
                &s,
                &mut map_handler,
            )
            .unwrap();

        let c = c.as_any().downcast_ref::<WCModuleConfig>().unwrap();

        assert_eq!(c.p0.as_ref().unwrap().get_manual().state, true);

        let sended_config = r.recv_timeout(Duration::from_millis(100)).unwrap();
        assert_eq!(*sended_config.data.get(0).unwrap(), 1);
        assert_eq!(*sended_config.data.get(1).unwrap(), 255);
    }
}
