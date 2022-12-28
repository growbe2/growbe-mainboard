use crate::modulestate::actor::get_owner;
use crate::modulestate::relay::configure::{
    authorize_relay_change, change_ownership_relay_property, configure_relay,
};
use crate::modulestate::relay::get_outlet_data;
use crate::modulestate::relay::physical_relay::ActionPortUnion;
use crate::modulestate::relay::BatchRelay;
use crate::protos::module::{Actor, RelayModuleConfig, RelayModuleData, RelayOutletConfig};
use crate::set_property;
use protobuf::Message;
use protobuf::SingularPtrField;

use crate::socket::ss::SenderPayload;
pub struct AAPValidator {
    pub actors_property: std::collections::HashMap<String, Actor>,
    pub previous_config: RelayModuleConfig,
    pub clear_actor: bool,
}

impl AAPValidator {
    pub fn new() -> AAPValidator {
        return AAPValidator {
            clear_actor: false,
            actors_property: std::collections::HashMap::new(),
            previous_config: RelayModuleConfig::new(),
        };
    }
}

impl crate::modulestate::interface::ModuleValue for RelayModuleData {}

impl crate::modulestate::interface::ModuleValueParsable for RelayModuleData {}

impl crate::modulestate::interface::ModuleValueValidator for AAPValidator {
    fn edit_ownership(
        &mut self,
        config: Box<dyn protobuf::Message>,
        request: crate::protos::module::ModuleActorOwnershipRequest,
        actor: &crate::protos::module::Actor,
    ) -> Result<Box<dyn protobuf::Message>, crate::modulestate::interface::ModuleError> {
        let mut config = config.as_any().downcast_ref::<RelayModuleConfig>().unwrap().clone();

        change_ownership_relay_property(
            "p0",
            &request.property,
            config.mut_p0(),
            self.previous_config.get_p0(),
            &actor,
        )?;
        change_ownership_relay_property(
            "p1",
            &request.property,
            config.mut_p1(),
            self.previous_config.get_p1(),
            &actor,
        )?;
        change_ownership_relay_property(
            "p2",
            &request.property,
            config.mut_p2(),
            self.previous_config.get_p2(),
            &actor,
        )?;

       change_ownership_relay_property(
            "p3",
            &request.property,
            config.mut_p3(),
            self.previous_config.get_p3(),
            &actor,
        )?;

       change_ownership_relay_property(
            "p4",
            &request.property,
            config.mut_p4(),
            self.previous_config.get_p4(),
            &actor,
        )?;

       change_ownership_relay_property(
            "p5",
            &request.property,
            config.mut_p5(),
            self.previous_config.get_p5(),
            &actor,
        )?;

       change_ownership_relay_property(
            "p6",
            &request.property,
            config.mut_p6(),
            self.previous_config.get_p6(),
            &actor,
        )?;

       change_ownership_relay_property(
            "p7",
            &request.property,
            config.mut_p7(),
            self.previous_config.get_p7(),
            &actor,
        )?;

        self.clear_actor = true;

        let config = Box::new(config);
        return Ok(config);
    }

    fn convert_to_value(
        &mut self,
        value_event: &crate::comboard::imple::interface::ModuleValueValidationEvent,
    ) -> Result<
        Box<dyn crate::modulestate::interface::ModuleValueParsable>,
        crate::modulestate::interface::ModuleError,
    > {
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
            data.timestamp = std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i32;
            return Ok(Box::new(data));
        } else {
            return Err(crate::modulestate::interface::ModuleError::new()
                .message("buffer not long enought".into()));
        }
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
        let mut config: Box<RelayModuleConfig> = if t == "AAP" {
            Box::new(RelayModuleConfig::parse_from_bytes(&data).map_err(|_e| {
                crate::modulestate::interface::ModuleError::new().message(_e.to_string())
            })?)
        } else {
            let property = t.split(":").last();
            if property.is_none() {
                Box::new(RelayModuleConfig::new())
            } else {
                let property = property.unwrap();
                let relay_outlet = RelayOutletConfig::parse_from_bytes(&data).map_err(|_e| {
                    crate::modulestate::interface::ModuleError::new()
                        .message("failed to parse relay outlet config".into())
                })?;

                let mut config = self.previous_config.clone();
                set_property!(
                    config,
                    property,
                    relay_outlet,
                    p0,
                    p1,
                    p2,
                    p3,
                    p4,
                    p5,
                    p6,
                    p7
                );

                Box::new(config)
            }
        };

        let buffer = [255; 8];
        let mut batch_relay = crate::modulestate::relay::physical_relay::BatchPhysicalRelay {
            action_port: ActionPortUnion::new_port(0),
            buffer: [255; 8],
            port,
            auto_send: false,
            sender: sender_comboard_config.clone(),
        };
        authorize_relay_change(
            config.has_p0(),
            config.get_p0(),
            self.previous_config.has_p0(),
            self.previous_config.get_p0(),
            &actor,
        )?;
        authorize_relay_change(
            config.has_p1(),
            config.get_p1(),
            self.previous_config.has_p1(),
            self.previous_config.get_p1(),
            &actor,
        )?;
        authorize_relay_change(
            config.has_p2(),
            config.get_p2(),
            self.previous_config.has_p2(),
            self.previous_config.get_p2(),
            &actor,
        )?;
        authorize_relay_change(
            config.has_p3(),
            config.get_p3(),
            self.previous_config.has_p3(),
            self.previous_config.get_p3(),
            &actor,
        )?;
        authorize_relay_change(
            config.has_p4(),
            config.get_p4(),
            self.previous_config.has_p4(),
            self.previous_config.get_p4(),
            &actor,
        )?;
        authorize_relay_change(
            config.has_p5(),
            config.get_p5(),
            self.previous_config.has_p5(),
            self.previous_config.get_p5(),
            &actor,
        )?;
        authorize_relay_change(
            config.has_p6(),
            config.get_p6(),
            self.previous_config.has_p6(),
            self.previous_config.get_p6(),
            &actor,
        )?;
        authorize_relay_change(
            config.has_p7(),
            config.get_p7(),
            self.previous_config.has_p7(),
            self.previous_config.get_p7(),
            &actor,
        )?;

        configure_relay(
            config.has_p0(),
            config.mut_p0(),
            self.previous_config.has_p0(),
            self.previous_config.get_p0(),
            &mut batch_relay,
            map_handler,
            &actor,
            self.clear_actor,
        )?;

        batch_relay.action_port.port = 1;
        configure_relay(
            config.has_p1(),
            config.mut_p1(),
            self.previous_config.has_p1(),
            self.previous_config.get_p1(),
            &mut batch_relay,
            map_handler,
            &actor,
            self.clear_actor,
        )?;

        batch_relay.action_port.port = 2;
        configure_relay(
            config.has_p2(),
            config.mut_p2(),
            self.previous_config.has_p2(),
            self.previous_config.get_p2(),
            &mut batch_relay,
            map_handler,
            &actor,
            self.clear_actor,
        )?;

        batch_relay.action_port.port = 3;
        configure_relay(
            config.has_p3(),
            config.mut_p3(),
            self.previous_config.has_p3(),
            self.previous_config.get_p3(),
            &mut batch_relay,
            map_handler,
            &actor,
            self.clear_actor,
        )?;

        batch_relay.action_port.port = 4;
        configure_relay(
            config.has_p4(),
            config.mut_p4(),
            self.previous_config.has_p4(),
            self.previous_config.get_p4(),
            &mut batch_relay,
            map_handler,
            &actor,
            self.clear_actor,
        )?;

        batch_relay.action_port.port = 5;
        configure_relay(
            config.has_p5(),
            config.mut_p5(),
            self.previous_config.has_p5(),
            self.previous_config.get_p5(),
            &mut batch_relay,
            map_handler,
            &actor,
            self.clear_actor,
        )?;

        batch_relay.action_port.port = 6;
        configure_relay(
            config.has_p6(),
            config.mut_p6(),
            self.previous_config.has_p6(),
            self.previous_config.get_p6(),
            &mut batch_relay,
            map_handler,
            &actor,
            self.clear_actor,
        )?;

        batch_relay.action_port.port = 7;
        configure_relay(
            config.has_p7(),
            config.mut_p7(),
            self.previous_config.has_p7(),
            self.previous_config.get_p7(),
            &mut batch_relay,
            map_handler,
            &actor,
            self.clear_actor,
        )?;

        batch_relay.execute().unwrap();

        self.previous_config = *config.clone();
        self.clear_actor = false;

        println!("Applying config : {:?}", self.previous_config);

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
        let current = current.as_any().downcast_ref::<RelayModuleData>().unwrap();
        let last = last.as_any().downcast_ref::<RelayModuleData>().unwrap();

        if current.p0.as_ref().unwrap().state != last.p0.as_ref().unwrap().state {
            return (true, vec![]);
        } else if current.p1.as_ref().unwrap().state != last.p1.as_ref().unwrap().state {
            return (true, vec![]);
        } else if current.p2.as_ref().unwrap().state != last.p2.as_ref().unwrap().state {
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

    use std::{collections::HashMap, sync::Arc, time::Duration};

    use tokio::sync::mpsc::channel;
    use tokio_util::sync::CancellationToken;

    use crate::modulestate::actor::new_actor;
    use crate::{
        comboard::imple::channel::ModuleConfig, modulestate::interface::ModuleValueValidator,
        protos::module::ManualConfig,
    };

    use crate::wait_async;
    use tokio::select;

    use super::*;

    #[test]
    fn module_aap_apply_full_config() {
        let mut validator = AAPValidator::new();
        let (s, _r) = channel::<ModuleConfig>(5);
        let config = RelayModuleConfig::new();
        let mut map_handler: HashMap<String, CancellationToken> = HashMap::new();
        let actor = new_actor("a", crate::protos::module::ActorType::MANUAL_USER_ACTOR);

        validator
            .apply_parse_config(
                0,
                "AAP",
                Arc::new(config.write_to_bytes().unwrap()),
                &s,
                &mut map_handler,
                actor,
            )
            .unwrap();
    }

    #[tokio::test]
    async fn module_aap_apply_partial_config() {
        let mut validator = AAPValidator::new();
        let (s, mut r) = channel::<ModuleConfig>(5);
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
                "AAP:p0",
                Arc::new(config.write_to_bytes().unwrap()),
                &s,
                &mut map_handler,
                actor,
            )
            .unwrap();

        let c = c.as_any().downcast_ref::<RelayModuleConfig>().unwrap();

        assert_eq!(c.p0.as_ref().unwrap().get_manual().state, true);

        let sended_config = wait_async!(r.recv(), Duration::from_millis(100), None).unwrap();
        assert_eq!(*sended_config.data.get(0).unwrap(), 1);
        assert_eq!(*sended_config.data.get(1).unwrap(), 255);
    }

    #[tokio::test]
    async fn module_aap_apply_partial_config_same_owner() {
        let mut validator = AAPValidator::new();
        let (s, mut r) = channel::<ModuleConfig>(5);
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
                "AAP:p0",
                Arc::new(config.write_to_bytes().unwrap()),
                &s,
                &mut map_handler,
                actor.clone(),
            )
            .unwrap();

        let (c, _) = validator
            .apply_parse_config(
                0,
                "AAP:p0",
                Arc::new(config.write_to_bytes().unwrap()),
                &s,
                &mut map_handler,
                actor.clone(),
            )
            .unwrap();

        let c = c.as_any().downcast_ref::<RelayModuleConfig>().unwrap();

        assert_eq!(c.p0.as_ref().unwrap().get_manual().state, true);

        let sended_config = wait_async!(r.recv(), Duration::from_millis(100), None).unwrap();
        assert_eq!(*sended_config.data.get(0).unwrap(), 1);
        assert_eq!(*sended_config.data.get(1).unwrap(), 255);
    }

    #[tokio::test]
    async fn module_aap_apply_partial_already_owned() {
        let mut validator = AAPValidator::new();
        let (s, mut r) = channel::<ModuleConfig>(5);
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
                "AAP:p0",
                Arc::new(config.write_to_bytes().unwrap()),
                &s,
                &mut map_handler,
                actor.clone(),
            )
            .unwrap();

        // switch actor but dont send property is ok
        let new_actor = new_actor("ab", crate::protos::module::ActorType::ENV_CONTROLLER_ACTOR);
        let mut config = RelayOutletConfig::new();
        let manual = ManualConfig {
            state: false,
            ..Default::default()
        };
        config.set_manual(manual);

        let result = validator.apply_parse_config(
            0,
            "AAP:p0",
            Arc::new(config.write_to_bytes().unwrap()),
            &s,
            &mut map_handler,
            new_actor.clone(),
        );

        assert_eq!(result.is_err(), true);
    }
}
