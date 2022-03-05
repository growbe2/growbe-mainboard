use tokio_util::sync::CancellationToken;

use crate::protos::module::{RelayOutletConfig, Actor, RelayOutletMode, AlarmConfig, CronItem, CycleConfig};

use super::Relay;


fn compare_cron(current: &CronItem, previous: &CronItem) -> bool {
    return current.minute == previous.minute && current.hour == previous.hour;
}

fn compare_alarm(current: &AlarmConfig, previous: &AlarmConfig) -> bool {
    return compare_cron(current.get_begining(), previous.get_begining()) && compare_cron(current.get_end(), previous.get_end());
}

fn compare_cycle(current: &CycleConfig, previous: &CycleConfig) -> bool {
    return current.runningTime == previous.runningTime && current.waitingTime == previous.waitingTime;
}

pub fn configure_relay(
    has_field: bool,
    config: &RelayOutletConfig,
    has_field_previous: bool,
    prev_config: &RelayOutletConfig,
    relay: &mut impl Relay,
    map_handler: & mut std::collections::HashMap<String, CancellationToken>,
    _previous_owner: std::option::Option<&Actor>
) -> std::option::Option<Actor> {

    if has_field {
        let id = relay.id();
        let previous_handler = map_handler.get(&id);

        if has_field_previous {
            if prev_config.mode == config.mode {
                // Match pour regarder si ca la changer
                match config.mode {
                    RelayOutletMode::MANUAL => {
                        if config.get_manual().state == prev_config.get_manual().state {
                            return None;
                        }
                    },
                    RelayOutletMode::ALARM => {
                        if compare_alarm( config.get_alarm(), prev_config.get_alarm()) {
                            return None;
                        }
                    },
                    RelayOutletMode::CYCLE => {
                        if compare_cycle(config.get_cycle(), prev_config.get_cycle()) {
                            return None;
                        }
                    },
                    _ => {
                    }
                }
            }
        }

        if previous_handler.is_some() {
            previous_handler.unwrap().cancel();
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
                    super::duration::set_duration_relay(manual_config.duration as u64, relay , token.clone());
                    map_handler.insert(
                        id,
                        token
                    );
                }
                return None;
            },
            RelayOutletMode::ALARM => {
                let token = CancellationToken::new();
                let config = config.alarm.as_ref().unwrap();
                super::alarm::set_alarm_relay(relay, config, token.clone());
                map_handler.insert(
                    id,
                    token
                );
                return None;
            },
            RelayOutletMode::CYCLE => {
                let token = CancellationToken::new();
                let config = config.cycle.as_ref().unwrap();
                super::cycle::set_cycle_relay(relay, config, token.clone());
                map_handler.insert(
                    id,
                    token
                );
                return None;
            },
            RelayOutletMode::VIRTUAL => {
                // i'm a virtual relay , i do nothing here put more elsewhere
            }
        }
    }
    return None;
}
