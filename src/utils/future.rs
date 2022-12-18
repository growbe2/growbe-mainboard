#[macro_export]
macro_rules! wait_async {
    ($recever: expr, $duration: expr, $none: expr) => {
        tokio::select! {
            v = $recever => {
                v
            },
            _ = tokio::time::sleep($duration) => {
                $none
            }
        }
    };
}
