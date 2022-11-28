pub mod tungstenite;

pub fn get_ws_comboard(config: String) -> Box<tungstenite::WSComboardClient> {
    return Box::new(tungstenite::WSComboardClient {
        config_comboard: crate::comboard::imple::interface::ComboardClientConfig { config: config },
    });
}
