use crate::modulestate::relay::configure::{
    authorize_relay_change, change_ownership_relay_property, configure_relay,
};
use crate::modulestate::relay::BatchRelay;
use crate::modulestate::relay::{get_outlet_data, physical_relay::ActionPortUnion};
use crate::protos::module::{Actor, RelayOutletConfig, WCModuleConfig, WCModuleData};
use protobuf::Message;
use protobuf::SingularPtrField;
use std::collections::HashMap;

use crate::{
    authorize_relay, authorize_relays, change_ownership_relay, change_ownership_relays,
    configure_relay, configure_relays, set_property,
};

pub struct AABValidator {
    pub actors_property: HashMap<String, Actor>,
    pub previous_config: WCModuleConfig,
    pub previous_value: WCModuleData,
    pub clear_actor: bool,
}

impl AABValidator {
    pub fn new() -> AABValidator {
        return AABValidator {
            clear_actor: false,
            actors_property: HashMap::new(),
            previous_config: WCModuleConfig::new(),
            previous_value: WCModuleData::new(),
        };
    }
}

impl crate::modulestate::interface::ModuleValue for WCModuleData {}

impl crate::modulestate::interface::ModuleValueParsable for WCModuleData {}

impl crate::modulestate::interface::ModuleValueValidator for AABValidator {
    fn edit_ownership(
        &mut self,
        config: Box<dyn protobuf::Message>,
        request: crate::protos::module::ModuleActorOwnershipRequest,
        actor: &crate::protos::module::Actor,
    ) -> Result<Box<dyn protobuf::Message>, crate::modulestate::interface::ModuleError> {
        return change_ownership_relays!(
            self,
            config,
            WCModuleConfig,
            self.previous_config,
            actor,
            request.property,
            p0,
            p1,
            p2,
            drain,
            pump0,
            pump1,
            pump2,
            pump3
        );
    }

    fn convert_to_value(
        &mut self,
        value_event: &crate::comboard::imple::interface::ModuleValueValidationEvent,
    ) -> Result<
        Box<dyn crate::modulestate::interface::ModuleValueParsable>,
        crate::modulestate::interface::ModuleError,
    > {
        let mut data = crate::protos::module::WCModuleData::new();
        data.p0 = get_outlet_data(value_event.buffer[0], self.previous_value.get_p0());
        data.p1 = get_outlet_data(value_event.buffer[1], self.previous_value.get_p1());
        data.p2 = get_outlet_data(value_event.buffer[2], self.previous_value.get_p2());
        data.drain = get_outlet_data(value_event.buffer[3], self.previous_value.get_drain());
        data.pump0 = get_outlet_data(value_event.buffer[4], self.previous_value.get_pump0());
        data.pump1 = get_outlet_data(value_event.buffer[5], self.previous_value.get_pump1());
        data.pump2 = get_outlet_data(value_event.buffer[6], self.previous_value.get_pump2());
        data.pump3 = get_outlet_data(value_event.buffer[7], self.previous_value.get_pump3());
        data.timestamp = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i32;

        self.previous_value = data.clone();
        return Ok(Box::new(data));
    }

    fn remove_config(
        &mut self,
        _actor: crate::protos::module::Actor,
    ) -> Result<(), crate::modulestate::interface::ModuleError> {
        return Ok(());
    }

    fn apply_parse_config(
        &mut self,
        port: i32,
        t: &str,
        data: std::sync::Arc<Vec<u8>>,
        sender_comboard_config: &tokio::sync::mpsc::Sender<
            crate::comboard::imple::channel::ModuleConfig,
        >,
        map_handler: &mut std::collections::HashMap<String, tokio_util::sync::CancellationToken>,
        actor: crate::protos::module::Actor,
    ) -> Result<
        (
            Box<dyn protobuf::Message>,
            crate::comboard::imple::channel::ModuleConfig,
        ),
        crate::modulestate::interface::ModuleError,
    > {
        let mut config: Box<WCModuleConfig> = if t == "AAB" {
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
                let mut relay_outlet = RelayOutletConfig::parse_from_bytes(&data)
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

        let mut batch_relay = crate::modulestate::relay::physical_relay::BatchPhysicalRelay {
            action_port: ActionPortUnion::new_port(0),
            buffer: [255; 8],
            auto_send: false,
            port,
            sender: sender_comboard_config.clone(),
        };

        authorize_relays!(
            config,
            self.previous_config,
            actor,
            p0,
            p1,
            p2,
            drain,
            pump0,
            pump1,
            pump2,
            pump3
        );

        configure_relays!(
            config,
            self.previous_config,
            actor,
            batch_relay,
            map_handler,
            self.clear_actor,
            p0,
            p1,
            p2,
            drain,
            pump0,
            pump1,
            pump2,
            pump3
        );

        batch_relay.action_port.port -= 1;

        batch_relay.execute().unwrap();

        self.previous_config = *config.clone();
        self.clear_actor = false;

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
        _sender_response: tokio::sync::oneshot::Sender<crate::protos::message::ActionResponse>,
        _sender_socket: &tokio::sync::mpsc::Sender<crate::socket::ss::SenderPayload>,
        _actor: crate::protos::module::Actor,
    ) -> Result<
        Option<Vec<crate::modulestate::interface::ModuleStateCmd>>,
        crate::modulestate::interface::ModuleError,
    > {
        return Ok(None);
    }
}

#[cfg(test)]
mod tests {

    use std::{collections::HashMap, sync::Arc};

    use tokio::select;
    use tokio::sync::mpsc::channel;
    use tokio_util::sync::CancellationToken;

    use crate::{
        cast,
        comboard::imple::channel::ModuleConfig,
        modulestate::{actor::new_actor, interface::ModuleValueValidator},
        protos::module::ManualConfig,
        wait_async,
    };

    use super::*;

    #[test]
    fn module_aab_apply_full_config() {
        let mut validator = AABValidator::new();
        let (s, _r) = channel::<ModuleConfig>(10);
        let mut config = WCModuleConfig::new();
        let mut p0 = RelayOutletConfig::new();
        let mut manual = ManualConfig::new();
        manual.state = true;
        p0.set_manual(manual);
        config.set_p0(p0);

        let mut map_handler: HashMap<String, CancellationToken> = HashMap::new();
        let actor = new_actor("a", crate::protos::module::ActorType::MANUAL_USER_ACTOR);

        let new_config = validator
            .apply_parse_config(
                0,
                "AAB",
                Arc::new(config.write_to_bytes().unwrap()),
                &s,
                &mut map_handler,
                actor,
            )
            .unwrap()
            .0;

        let new_config = cast!(new_config, WCModuleConfig);

        assert_ne!(new_config.get_p0().get_timestamp(), 0);
        assert_eq!(new_config.get_p1().get_timestamp(), 0);
    }

    #[tokio::test]
    async fn module_aab_apply_partial_config() {
        let mut validator = AABValidator::new();
        let (s, mut r) = channel::<ModuleConfig>(10);
        let mut config = RelayOutletConfig::new();
        let manual = ManualConfig {
            state: true,
            ..Default::default()
        };
        config.set_manual(manual);
        let mut map_handler: HashMap<String, CancellationToken> = HashMap::new();
        let actor = new_actor("a", crate::protos::module::ActorType::MANUAL_USER_ACTOR);

        let (c, _) = validator
            .apply_parse_config(
                0,
                "AAB:p0",
                Arc::new(config.write_to_bytes().unwrap()),
                &s,
                &mut map_handler,
                actor,
            )
            .unwrap();

        let c = cast!(c, WCModuleConfig);

        assert_eq!(c.p0.as_ref().unwrap().get_manual().state, true);
        assert_ne!(c.p0.as_ref().unwrap().get_timestamp(), 0);

        let sended_config =
            wait_async!(r.recv(), std::time::Duration::from_millis(100), None).unwrap();
        assert_eq!(*sended_config.data.get(0).unwrap(), 1);
        assert_eq!(*sended_config.data.get(1).unwrap(), 255);
    }
}
