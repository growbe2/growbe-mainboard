
use tokio::select;
use tokio_util::sync::CancellationToken;

pub fn set_duration_relay(
    duration: u64,
    relay: &mut impl super::Relay,
    cancellation_token: CancellationToken,
) -> tokio::task::JoinHandle<()> {
    log::debug!("Creating duration task");

    let mut relay = relay.clone_me();
    return tokio::task::spawn(async move {
        log::debug!("Start duration timeout");
        select! {
            _ = cancellation_token.cancelled() => {
                log::debug!("cancellation of duration timeout");
            },
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(duration)) => {
                log::debug!("End duration timeout");
                relay.set_state(0).unwrap();
            }
        }
    });
}
