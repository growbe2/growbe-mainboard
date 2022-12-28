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

fn is_changing(
    config: &RelayOutletConfig,
    prev_config: &RelayOutletConfig,
) -> bool {
            if prev_config.mode == config.mode {
                // Match pour regarder si ca la changer
                match config.mode {
                    RelayOutletMode::MANUAL => {
                        if config.get_manual().state == prev_config.get_manual().state {
                            return true;
                        }
                    }
                    RelayOutletMode::ALARM => {
                        if compare_alarm(config.get_alarm(), prev_config.get_alarm()) {
                            return true;
                        }
                    }
                    RelayOutletMode::CYCLE => {
                        if compare_cycle(config.get_cycle(), prev_config.get_cycle()) {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
            return false;

}

pub fn authorize_relay_change(
    has_field: bool,
    config: &RelayOutletConfig,
    has_field_previous: bool,
    prev_config: &RelayOutletConfig,
    actor: &Actor,
) -> Result<(), ModuleError> {
    if actor.id != "handle_state" && has_field && has_field_previous && is_changing(config, prev_config) {
        if prev_config.get_actor_owner_id() != "" && prev_config.get_actor_owner_id() != actor.id {
            return Err(ModuleError::new().message(format!(
                "cant change property already owned by other actor : {} -> {} , {:?} {:?}",
                prev_config.get_actor_owner_id(),
                actor.get_id(),
                config,
                prev_config,
            )));
        }
    }
    return Ok(());
}

pub fn change_ownership_relay_property(
    property: &str,
    map: &HashMap<String, bool>,
    config: &mut RelayOutletConfig,
    previous_config: &RelayOutletConfig,
    actor: &Actor,
) -> Result<(), ModuleError> {
    if let Some(v) = map.get(property) {
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
    return Ok(());
}

pub fn configure_relay(
    has_field: bool,
    config: &mut RelayOutletConfig,
    has_field_previous: bool,
    prev_config: &RelayOutletConfig,
    relay: &mut impl Relay,
    map_handler: &mut std::collections::HashMap<String, CancellationToken>,
    actor: &Actor,
    clear_actor: bool,
) -> Result<(), ModuleError> {
    if has_field {
        let id = relay.id();
        let previous_handler = map_handler.get(&id);

        if has_field_previous {
            if prev_config.mode == config.mode {
                // Match pour regarder si ca la changer
                match config.mode {
                    RelayOutletMode::MANUAL => {
                        if config.get_manual().state == prev_config.get_manual().state {
                            return Ok(());
                        }
                    }
                    RelayOutletMode::ALARM => {
                        if compare_alarm(config.get_alarm(), prev_config.get_alarm()) {
                            return Ok(());
                        }
                    }
                    RelayOutletMode::CYCLE => {
                        if compare_cycle(config.get_cycle(), prev_config.get_cycle()) {
                            return Ok(());
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
                let manual_config = config.manual.as_ref().unwrap();
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
                return Ok(());
            }
            RelayOutletMode::ALARM => {
                let token = CancellationToken::new();
                let config = config.alarm.as_ref().unwrap();
                super::alarm::set_alarm_relay(relay, config, token.clone());
                map_handler.insert(id, token);
                return Ok(());
            }
            RelayOutletMode::CYCLE => {
                let token = CancellationToken::new();
                let config = config.cycle.as_ref().unwrap();
                super::cycle::set_cycle_relay(relay, config, token.clone());
                map_handler.insert(id, token);
                return Ok(());
            }
            RelayOutletMode::VIRTUAL => {
                // i'm a virtual relay , i do nothing here put more elsewhere
            }
        }
    }
    return Ok(());
}
