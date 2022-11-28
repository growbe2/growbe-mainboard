use crate::protos::board::LocalConnection;
use std::sync::mpsc::Sender;

impl crate::modulestate::interface::ModuleValue for crate::protos::board::LocalConnection {}
impl crate::modulestate::interface::ModuleValueParsable for crate::protos::board::LocalConnection {}

pub async fn task_local_connection(
    sender: Sender<(
        String,
        Box<dyn crate::modulestate::interface::ModuleValueParsable>,
    )>,
) -> () {
    let local_connection = get_local_connection();
    sender
        .send((String::from("/localconnection"), Box::new(local_connection)))
        .unwrap();
}

pub fn get_local_connection() -> LocalConnection {
    let mut local_connection = LocalConnection::new();

    local_connection.set_ipAddr(crate::plateform::net::get_ip_addr());

    local_connection.set_ssid(crate::plateform::wifi::get_currnet_ssid());

    local_connection.set_signalLevel(crate::plateform::wifi::get_curret_ssid_strength());

    return local_connection;
}
