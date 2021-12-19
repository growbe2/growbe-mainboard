use crate::protos::module::{AlarmConfig, RelayOutletConfig, RelayOutletData, RelayOutletMode, Actor};
use protobuf::SingularPtrField;
use chrono::Timelike;
use tokio::select;
use tokio_util::sync::CancellationToken;

fn f(i: &usize, x: &mut [u8], value: u8) {
    x[*i] = value;
}

fn set_duration_relay(
    action_port: usize,
    port: i32,
    duration: u64,
    clone_sender: std::sync::mpsc::Sender<crate::comboard::imple::interface::Module_Config>,
    cancellation_token: CancellationToken,
) -> tokio::task::JoinHandle<()> {
    log::debug!("Creating duration task");
    return tokio::task::spawn(async move {
        log::debug!("Start duration timeout");
        select! {
            _ = cancellation_token.cancelled() => {
                log::debug!("cancellation of duration timeout");
            },
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(duration)) => {
                log::debug!("End duration timeout");
                let mut buffer = [255; 8];
                f(&action_port, &mut buffer, 0);
                clone_sender.send(crate::comboard::imple::interface::Module_Config{
                    port: port,
                    buffer: buffer
                }).unwrap();
            }
        }
    });
}

fn set_alarm_relay(
    action_port: usize,
    port: i32,
    config: &AlarmConfig,
    clone_sender: std::sync::mpsc::Sender<crate::comboard::imple::interface::Module_Config>,
    cancellation_token: CancellationToken,
) -> tokio::task::JoinHandle<()> {

    let beginning = config.begining.as_ref().unwrap();
    let ending = config.end.as_ref().unwrap();

    let hr_day = beginning.hour;
    let min_day = beginning.minute;

    let hr_night = ending.hour;
    let min_night = ending.minute;

    let day_mins = hr_day*60+min_day;
    let night_mins = hr_night*60+min_night;

   
    return tokio::spawn(async move {
        log::debug!("starting alarm process for port {}, day_mins {} night_mins {}", port, day_mins, night_mins);

        // TODO : make a better implementation , this will cause issue on clock update, hour change
        loop {
            let curr_time = crate::mainboardstate::rtc::get_rtc().time();
            let current_minute: i32 = (curr_time.hour() * 60 + curr_time.minute()) as i32;

            let (timeout, day) = if current_minute < day_mins {
                ((day_mins - current_minute), false)
            } else if current_minute < night_mins {
                ((night_mins - current_minute), true)
            } else if current_minute >= night_mins {
               ((((24 * 60) - current_minute) + day_mins), false)
            } else {
                panic!("impossibru")
            };

            let mut buffer = [255; 8];
            let value = if day { 1 } else { 0 };
            f(&action_port, &mut buffer, value);

            clone_sender.send(crate::comboard::imple::interface::Module_Config{
                port: port,
                buffer: buffer
            }).unwrap();

            log::debug!("have to sleep for {} minute(s) from {}", timeout, current_minute);

            select! {
                _ = cancellation_token.cancelled() => {
                    log::debug!("cancellation of alarm");
                    return;
                },
                _ = tokio::time::sleep(tokio::time::Duration::from_secs((timeout * 60) as u64)) => {
                    log::debug!("End of timeout of alarm");
                }
            }
        }
    });
}

pub fn get_outlet_data(value: u8) -> SingularPtrField<RelayOutletData> {
    let mut data = RelayOutletData::new();
    if value == 0 {
        data.set_state(false);
    } else if value == 1 {
        data.set_state(true);
    }
    return SingularPtrField::some(data);
}

pub fn configure_relay(
    has_field: bool,
    action_port: usize,
    port: &i32,
    config: &RelayOutletConfig,
    buffer: & mut u8,
    sender_comboard_config: & std::sync::mpsc::Sender<crate::comboard::imple::interface::Module_Config>,
    map_handler: & mut std::collections::HashMap<i32, CancellationToken>,
    previous_owner: std::option::Option<&Actor>
) -> std::option::Option<Actor> {

    if has_field {
        let previous_handler = map_handler.get(&(action_port as i32));
        if previous_handler.is_some() {
            log::debug!("aborting previous handler for port {}", port);
            previous_handler.unwrap().cancel();
        }

        match config.mode {
            RelayOutletMode::MANUAL => {
                let manual_config = config.manual.as_ref().unwrap();
                if manual_config.state == true {
                    *buffer = 1;
                } else {
                    *buffer = 0;
                }

                if manual_config.duration > 0 {
                    let token = CancellationToken::new();
                    set_duration_relay(action_port, *port, manual_config.duration as u64, sender_comboard_config.clone(), token.clone());
                    map_handler.insert(
                       action_port as i32,
                       token
                    );
                }
                return None;
            },
            RelayOutletMode::ALARM => {
                let token = CancellationToken::new();
                let config = config.alarm.as_ref().unwrap();
                set_alarm_relay(action_port, *port, config, sender_comboard_config.clone(), token.clone());
                map_handler.insert(
                    action_port as i32,
                    token
                );
                return None;
            }
        }
    }
    return None;
}
