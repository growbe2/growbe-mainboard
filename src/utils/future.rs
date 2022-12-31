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

#[macro_export]
macro_rules! postpone {
    ($timeout: expr, $body: expr) => {{
        tokio::spawn(async {
            tokio::time::sleep($timeout).await;
            $body
        })
    }};
}

#[macro_export]
macro_rules! cast_enum {
    ($source: expr, $type: path) => {{
        if let $type(a) = $source {
            a
        } else {
            panic!("mismatch variant when cast to {}", stringify!($type));
        }
    }};
}
