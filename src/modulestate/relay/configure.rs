use std::collections::HashMap;

use tokio_util::sync::CancellationToken;

use crate::{
    mainboardstate::error::MainboardError,
    modulestate::interface::ModuleError,
    protos::module::{
        Actor, ActorType, AlarmConfig, CronItem, CycleConfig, ManualConfig, RelayOutletConfig,
        RelayOutletMode,
    },
};

use super::Relay;

fn compare_cron(current: &CronItem, previous: &CronItem) -> bool {
    return current.minute == previous.minute && current.hour == previous.hour;
}

fn compare_alarm(current: &AlarmConfig, previous: &AlarmConfig) -> bool {
    return compare_cron(current.get_begining(), previous.get_begining())
        && compare_cron(current.get_end(), previous.get_end());
}

fn compare_cycle(current: &CycleConfig, previous: &CycleConfig) -> bool {
    return current.runningTime == previous.runningTime
        && current.waitingTime == previous.waitingTime;
}

fn is_changing(config: &RelayOutletConfig, prev_config: &RelayOutletConfig) -> bool {
    if prev_config.mode == config.mode {
        // Match pour regarder si ca la changer
        match config.mode {
            RelayOutletMode::MANUAL => {
                return config.get_manual().state != prev_config.get_manual().state;
            }
            RelayOutletMode::ALARM => {
                return !compare_alarm(config.get_alarm(), prev_config.get_alarm());
            }
            RelayOutletMode::CYCLE => {
                return !compare_cycle(config.get_cycle(), prev_config.get_cycle());
            }
            _ => {
                return true;
            }
        }
    } else {
        return true;
    }
}

pub fn authorize_relay_change(
    config: Option<&RelayOutletConfig>,
    prev_config: Option<&RelayOutletConfig>,
    actor: &Actor,
) -> Result<(), ModuleError> {
    if actor.id != "handle_state" && config.is_some() && prev_config.is_some() {
        let config = config.unwrap();
        let prev_config = prev_config.unwrap();

        if is_changing(config, prev_config) {
            if prev_config.get_actor_owner_id() != ""
                && prev_config.get_actor_owner_type() != ActorType::MANUAL_USER_ACTOR
                && prev_config.get_actor_owner_id() != actor.id
            {
                return Err(ModuleError::new().message(format!(
                    "cant change property already owned by other actor : {} -> {} , {:?} {:?}",
                    prev_config.get_actor_owner_id(),
                    actor.get_id(),
                    config,
                    prev_config,
                )));
            }
        }
    }
    return Ok(());
}

pub fn change_ownership_relay_property(
    property: &str,
    map: &HashMap<String, bool>,
    config: Option<&RelayOutletConfig>,
    previous_config: Option<&RelayOutletConfig>,
    actor: &Actor,
) -> Result<RelayOutletConfig, ModuleError> {
    let mut config = config.unwrap().clone();
    if let Some(v) = map.get(property) {
        let previous_config = previous_config.unwrap();
        if previous_config.get_actor_owner_id() != ""
            && previous_config.get_actor_owner_id() != actor.id.as_str()
        {
            if previous_config.get_actor_owner_type() != ActorType::MANUAL_USER_ACTOR {
                return Err(ModuleError::new().message(format!("unauthorized")));
            }
        }
        // edit property
        if *v {
            println!("adding ownership");
            config.actor_owner_id = actor.id.clone();
            config.actor_owner_type = actor.field_type.clone();
        } else {
            println!("removing ownership");
            config.actor_owner_id = "".into();
            config.actor_owner_type = ActorType::MANUAL_USER_ACTOR;
        }
        let mut manual = ManualConfig::new();
        manual.state = false;
        config.set_manual(manual);
    }
    return Ok(config);
}

pub fn configure_relay(
    config: Option<&RelayOutletConfig>,
    prev_config: Option<&RelayOutletConfig>,
    relay: &mut impl Relay,
    map_handler: &mut std::collections::HashMap<String, CancellationToken>,
    actor: &Actor,
    clear_actor: bool,
) -> Result<RelayOutletConfig, ModuleError> {
    if config.is_some() {
        let mut config = config.unwrap().clone();
        let id = relay.id();
        let previous_handler = map_handler.get(&id);

        if prev_config.is_some() {
            let prev_config = prev_config.unwrap();
            if prev_config.mode == config.mode {
                // Match pour regarder si ca la changer
                match config.mode {
                    RelayOutletMode::MANUAL => {
                        if config.get_manual().state == prev_config.get_manual().state {
                            return Ok(prev_config.clone());
                        }
                    }
                    RelayOutletMode::ALARM => {
                        if compare_alarm(config.get_alarm(), prev_config.get_alarm()) {
                            return Ok(prev_config.clone());
                        }
                    }
                    RelayOutletMode::CYCLE => {
                        if compare_cycle(config.get_cycle(), prev_config.get_cycle()) {
                            return Ok(prev_config.clone());
                        }
                    }
                    _ => {}
                }
            }
        }

        if previous_handler.is_some() {
            previous_handler.unwrap().cancel();
        }

        if !clear_actor {
            if actor.id != "handle_state" {
                config.set_actor_owner_id(actor.id.clone());
                config.set_actor_owner_type(actor.field_type.clone());
            }
        }

        match config.mode {
            RelayOutletMode::MANUAL => {
                if let Some(manual_config) = config.manual.as_ref() {
                    if manual_config.state == true {
                        relay.set_state(1).unwrap();
                    } else {
                        relay.set_state(0).unwrap();
                    }

                    if manual_config.duration > 0 {
                        let token = CancellationToken::new();
                        super::duration::set_duration_relay(
                            manual_config.duration as u64,
                            relay,
                            token.clone(),
                        );
                        map_handler.insert(id, token);
                    }
                }
                return Ok(config);
            }
            RelayOutletMode::ALARM => {
                let token = CancellationToken::new();
                let aconfig = config.alarm.as_ref().unwrap();
                super::alarm::set_alarm_relay(relay, aconfig, token.clone());
                map_handler.insert(id, token);
                return Ok(config);
            }
            RelayOutletMode::CYCLE => {
                let token = CancellationToken::new();
                let rconfig = config.cycle.as_ref().unwrap();
                super::cycle::set_cycle_relay(relay, rconfig, token.clone());
                map_handler.insert(id, token);
                return Ok(config);
            }
            RelayOutletMode::VIRTUAL => {
                // i'm a virtual relay , i do nothing here put more elsewhere
            }
        }
    } else if prev_config.is_some() {
        return Ok(prev_config.unwrap().clone());
    }
    return Ok(RelayOutletConfig::new());
}

#[macro_export]
macro_rules! authorize_relay {
    ($self: ident, $prop: ident, $running: expr, $actor: expr) => {
        authorize_relay_change($self.$prop.as_ref(), $running.$prop.as_ref(), &$actor)?
    };
}

#[macro_export]
macro_rules! authorize_relays {
    ($self: ident, $running: expr, $actor: expr, $($prop: ident),+) => {
        $(
            authorize_relay!($self, $prop, $running, $actor);
        )+
    };
}

#[macro_export]
macro_rules! configure_relay {
    ($self: ident, $prop: ident, $running: expr, $actor: expr, $batch_relay: expr, $map_handler: expr, $clear_actor: expr) => {{
        let config = configure_relay(
            $self.$prop.as_ref(),
            $running.$prop.as_ref(),
            &mut $batch_relay,
            $map_handler,
            &$actor,
            $clear_actor,
        )?;

        $self.$prop = protobuf::SingularPtrField::some(config);
    }};
}

#[macro_export]
macro_rules! configure_relays {
    ($self: ident, $running: expr, $actor: expr, $batch_relay: expr, $map_handler: expr, $clear_actor: expr, $($prop: ident),+) => {
        $(
            configure_relay!($self, $prop, $running, $actor, $batch_relay, $map_handler, $clear_actor);
            $batch_relay.action_port.port += 1;
        )+
    };
}

#[macro_export]
macro_rules! change_ownership_relay {
    ($self: ident, $prop: ident, $running: expr, $actor: expr, $property: expr) => {
        let config = change_ownership_relay_property(
            stringify!($prop),
            &$property,
            $self.$prop.as_ref(),
            $running.$prop.as_ref(),
            &$actor,
        )?;
        $self.$prop = protobuf::SingularPtrField::some(config);
    };
}

#[macro_export]
macro_rules! change_ownership_relays {
    ($self: ident, $config: ident, $type: ident, $running: expr, $actor: expr, $property: expr, $($prop: ident),+) => {
        {
        let mut config = $config
            .as_any()
            .downcast_ref::<$type>()
            .unwrap()
            .clone();
        $(
            change_ownership_relay!(config, $prop, $running, $actor, $property);
        )+

        $self.clear_actor = true;

            Ok(Box::new(config))
        }

    };
}
