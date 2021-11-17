use std::sync::{Arc, Mutex};

use crate::protos::module::{RelayModuleConfig, WCModuleConfig};
use crate::store::database;
use protobuf::Message;

pub struct ModuleStateStore {
    pub conn: Arc<Mutex<rusqlite::Connection>>
}

impl ModuleStateStore {
    pub fn new(conn: Arc<Mutex<rusqlite::Connection>>) -> Self {
        return ModuleStateStore{conn};
    }


    pub fn get_module_config(
        &self,
        id: &String,
    ) -> Option<Box<dyn Message>> {
        let item = match id.chars().nth(2).unwrap() {
            'P' => self.get_module_config_inner(id, RelayModuleConfig::parse_from_bytes),
            //'B' => self.get_module_config_inner(id, WCModuleConfig::parse_from_bytes),
            _ => Result::Err(rusqlite::Error::InvalidQuery)
        };
        if item.is_ok() {
            return Some(Box::new(item.unwrap()));
        } else {
            return None;
        }
    }

    fn get_module_config_inner<T>(
        &self,
        id: &String,
        id2: for<'r> fn(&'r [u8]) -> std::result::Result<T, protobuf::ProtobufError>,
    ) -> Result<T, rusqlite::Error> {
        return database::get_field_from_table(
            &self.conn,
            "module_config",
            id,
            id2,
        );
    }

    pub fn store_module_config(
        &self,
        id: &String,
        config: Box<dyn protobuf::Message>,
    ) -> () {
        log::debug!("store module config {}", id);
        database::store_field_from_table(
            &self.conn,
            "module_config",
            id,
            config,
        );

    }
}