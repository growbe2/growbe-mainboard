use crate::protos::module::{AlarmConfig};
use chrono::Timelike;
use tokio::select;
use tokio_util::sync::CancellationToken;

pub fn set_alarm_relay(
    relay: &mut impl super::Relay,
    config: &AlarmConfig,
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


    let mut relay = relay.clone();
   
    return tokio::spawn(async move {
        log::debug!("starting alarm process for port {}, day_mins {} night_mins {}", relay.id(), day_mins, night_mins);

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

            let value = if day { 1 } else { 0 };
            relay.set_state(value).unwrap();

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
