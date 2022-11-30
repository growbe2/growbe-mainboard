use crate::protos::board::LocalConnection;
use std::sync::mpsc::Sender;

use super::error::MainboardError;

impl crate::modulestate::interface::ModuleValue for crate::protos::board::LocalConnection {}
impl crate::modulestate::interface::ModuleValueParsable for crate::protos::board::LocalConnection {}

pub async fn task_local_connection(
    sender: Sender<(
        String,
        Box<dyn crate::modulestate::interface::ModuleValueParsable>,
    )>,
) -> Result<(), MainboardError> {
    let local_connection = get_local_connection();
    return sender
        .send((String::from("/localconnection"), Box::new(local_connection)))
        .map_err(MainboardError::from_send_error);
}

pub fn get_local_connection() -> LocalConnection {
    let mut local_connection = LocalConnection::new();

    match crate::plateform::net::get_ip_addr() {
        Ok(ip) => local_connection.set_ipAddr(ip),
        Err(err) => log::error!("cannot get_ip_addr(): {:?}", err)
    }

    match crate::plateform::wifi::get_currnet_ssid() {
        Ok(ssid) => local_connection.set_ssid(ssid),
        Err(err) => log::error!("{:?}", err)
    }

    match crate::plateform::wifi::get_curret_ssid_strength() {
        Ok(strength) => local_connection.set_signalLevel(strength),
        Err(err) => log::error!("{:?}", err)
    }

    return local_connection;
}
