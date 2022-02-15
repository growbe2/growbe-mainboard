use crate::protos::module::CycleConfig;
use tokio_util::sync::CancellationToken;
use tokio::select;

pub fn set_cycle_relay(
    relay: &mut impl super::Relay,
    cycle: &CycleConfig,
    cancellation_token: CancellationToken,
) -> tokio::task::JoinHandle<()> {

    let mut relay = relay.clone_me();
    
    let waiting_time = tokio::time::Duration::from_secs(cycle.waitingTime as u64);
    let running_time = tokio::time::Duration::from_secs(cycle.runningTime as u64);

    let mut current_timeout: tokio::time::Duration = waiting_time;
    let mut is_waiting = false;

    return tokio::spawn(async move {
        log::debug!(
            "starting cycling process for port {}, running time {:?} , waiting time {:?}",
            relay.id(),
            waiting_time,
            running_time,
        );

        loop {
             if is_waiting {
                 relay.set_state(0).unwrap();
                 current_timeout = waiting_time;
                 log::debug!("waiting cycle on relay {} for {:?}", relay.id(), current_timeout)
             } else {
                 relay.set_state(1).unwrap();
                 current_timeout = running_time;
                 log::debug!("running cycle on relay {} for {:?}", relay.id(), current_timeout)
             }
             select! {
                _ = cancellation_token.cancelled() => {
                    log::debug!("cancellation of cycle");
                    return;
                },
                _ = tokio::time::sleep(current_timeout) => {
                    is_waiting = !is_waiting;
                }
            }
        }
    });
}