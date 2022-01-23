use std::sync::{Arc, Mutex};

use crate::protos::module::{RelayModuleConfig, WCModuleConfig, SOILModuleConfig};
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
            'P' => {
                let result = self.get_module_config_inner(id, RelayModuleConfig::parse_from_bytes);
                if let Ok(d) = result {
                    Ok(Box::new(d) as Box<dyn Message>)
                } else { Err(super::interface::ModuleError::new()) }
            },
            'B' => {
                let result = self.get_module_config_inner(id, WCModuleConfig::parse_from_bytes);
                log::debug!("{:?}", result);
                if let Ok(d) = result {
                    Ok(Box::new(d) as Box<dyn Message>)
                } else { Err(super::interface::ModuleError::new())}
            },
            'S' => {
                let result = self.get_module_config_inner(id, SOILModuleConfig::parse_from_bytes);
                if let Ok(config) = result {
                    Ok(Box::new(config) as Box<dyn Message>)
                } else { Err(super::interface::ModuleError::new())}
            },
            _ => Err(super::interface::ModuleError::new())
        };
        if item.is_ok() {
            return Some(item.unwrap());
        } else {
            log::debug!("{}", item.unwrap_err());
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
            "config",
            config,
        );

    }
}