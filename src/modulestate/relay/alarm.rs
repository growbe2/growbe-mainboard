use crate::protos::module::AlarmConfig;
use chrono::Timelike;
use tokio::select;
use tokio_util::sync::CancellationToken;

fn get_current_min() -> i32 {
    let curr_time = crate::mainboardstate::rtc::get_rtc().time();
    return (curr_time.hour() * 60 + curr_time.minute()) as i32;
}

fn get_timeout_alarm(current_mins: i32, day_mins: i32, night_mins: i32) -> (i32, bool) {
    return if night_mins < day_mins {
        if current_mins < night_mins {
            (night_mins - current_mins, true)
        } else if current_mins >= night_mins && current_mins < day_mins {
            (day_mins - current_mins, false)
        } else {
            ((1500 - current_mins) + night_mins, true)
        }
    } else if current_mins < day_mins {
        ((day_mins - current_mins), false)
    } else if current_mins < night_mins {
        ((night_mins - current_mins), true)
    } else {
        ((((24 * 60) - current_mins) + day_mins), false)
    }
}

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

    let day_mins = hr_day * 60 + min_day;
    let night_mins = hr_night * 60 + min_night;

    let mut relay = relay.clone_me();
    return tokio::spawn(async move {
        log::debug!(
            "starting alarm process for port {}, day_mins {} night_mins {}",
            relay.id(),
            day_mins,
            night_mins
        );

        loop {

            let current_minute = get_current_min();
            let (timeout, day) = get_timeout_alarm(current_minute, day_mins, night_mins);
            let value = if day { 1 } else { 0 };
            relay.set_state(value).unwrap();

            log::debug!(
                "have to sleep for {} minute(s) from {}",
                timeout,
                current_minute
            );

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


#[cfg(test)]
mod tests {

    use super::*;

    #[test] 
    fn valid_alarm_values() {
        let values: Vec<(i32,i32,i32,i32,bool)> = vec![
            // OPEN , CLOSE , CURRENT , EXPECTED SLEEP , IS_DAY
            (600, 800, 700, 100, true),
            (360, 0, 800, 700, true),
            (360, 0, 100, 260, false),
            (360, 100, 800, 800, true)
        ];

        for value in values.iter() {
            let (timeout, day) = get_timeout_alarm(value.2, value.0, value.1);

            assert_eq!(timeout, value.3);
            assert_eq!(day, value.4);
        }
   }

}