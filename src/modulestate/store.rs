use std::sync::{Arc, Mutex};

use crate::mainboardstate::error::MainboardError;
use crate::protos::module::{
    ComputerStreamingConfig, PhoneStreamingConfig, RelayModuleConfig, SOILModuleConfig,
    WCModuleConfig,
};
use crate::store::database;
use protobuf::Message;

pub struct ModuleStateStore {
    pub conn: Arc<Mutex<rusqlite::Connection>>,
}

impl ModuleStateStore {
    pub fn new(conn: Arc<Mutex<rusqlite::Connection>>) -> Self {
        return ModuleStateStore { conn };
    }

    pub fn get_module_config(&self, id: &String) -> Option<Box<dyn Message>> {
        let module_type = &id[..3];
        let item = match module_type {
            "AAP" => {
                let result = self.get_module_config_inner(id, RelayModuleConfig::parse_from_bytes);
                if let Ok(d) = result {
                    Ok(Box::new(d) as Box<dyn Message>)
                } else {
                    Err(super::interface::ModuleError::new())
                }
            }
            "AAB" => {
                let result = self.get_module_config_inner(id, WCModuleConfig::parse_from_bytes);
                if let Ok(d) = result {
                    Ok(Box::new(d) as Box<dyn Message>)
                } else {
                    Err(super::interface::ModuleError::new())
                }
            }
            "AAS" => {
                let result = self.get_module_config_inner(id, SOILModuleConfig::parse_from_bytes);
                if let Ok(config) = result {
                    Ok(Box::new(config) as Box<dyn Message>)
                } else {
                    Err(super::interface::ModuleError::new())
                }
            }
            "PCS" => {
                let result =
                    self.get_module_config_inner(id, PhoneStreamingConfig::parse_from_bytes);
                if let Ok(config) = result {
                    Ok(Box::new(config) as Box<dyn Message>)
                } else {
                    Err(super::interface::ModuleError::new())
                }
            }
            "CCS" => {
                let result =
                    self.get_module_config_inner(id, ComputerStreamingConfig::parse_from_bytes);
                if let Ok(config) = result {
                    Ok(Box::new(config) as Box<dyn Message>)
                } else {
                    Err(super::interface::ModuleError::new())
                }
            }
            _ => Err(super::interface::ModuleError::new()),
        };
        if item.is_ok() {
            return Some(item.unwrap());
        } else {
            log::debug!("{}", item.unwrap_err());
            return None;
        }
    }

    pub fn delete_module_config(&self, id: &str) -> Result<(), MainboardError> {
        return database::store_delete_key(&self.conn, "module_config", id);
    }

    fn get_module_config_inner<T>(
        &self,
        id: &String,
        id2: for<'r> fn(&'r [u8]) -> std::result::Result<T, protobuf::ProtobufError>,
    ) -> Result<T, MainboardError> {
        return database::get_field_from_table(&self.conn, "module_config", id, id2);
    }

    pub fn store_module_config(&self, id: &String, config: Box<dyn protobuf::Message>) -> Result<(), MainboardError> {
        log::debug!("store module config {}", id);
        return database::store_field_from_table(&self.conn, "module_config", id, "config", config);
    }
}


#[cfg(test)]
mod tests {

    use super::*;
    use crate::{store::database::nbr_entry, protos::module::SOILProbeConfig};
    use std::sync::{Arc, Mutex};


    fn module_id() -> String {
        "AAP0000003".to_string()
    }

    fn get_store() -> ModuleStateStore {
        let conn_database = Arc::new(
            Mutex::new(crate::store::database::init(Some("./database_test_modulestate.sqlite".to_string()))));
        let store = ModuleStateStore::new(conn_database);
        clear_store(&store);
        store
    }

    fn clear_store(store: &ModuleStateStore) {
        store
            .conn
            .lock()
            .unwrap()
            .execute("DELETE FROM module_config", [])
            .unwrap();
    }

    #[test]
    fn store_module_config_not_existing() {
        let store = get_store();

        let config_opt = store.get_module_config(&module_id());

        assert_eq!(config_opt.is_none(), true);
    }

    #[test]
    fn store_module_config_existing() {
        let store = get_store();

        let config = RelayModuleConfig::new();
        store.store_module_config(&module_id(), Box::new(config)).unwrap();

        store.get_module_config(&module_id()).unwrap();

        assert_eq!(nbr_entry(&store.conn, "module_config").unwrap(), 1);
    }

    #[test]
    fn store_module_config_wrong_type() {
        let store = get_store();

        let mut config = SOILModuleConfig::new();
        let mut probe = SOILProbeConfig::new();
        probe.set_low(300);
        probe.set_high(800);
        config.set_p0(probe);
        store.store_module_config(&module_id(), Box::new(config)).unwrap();

        store.delete_module_config(&module_id()).unwrap();

        assert_eq!(store.get_module_config(&module_id()).is_none(), true);
    }

    #[test]
    fn store_module_deleting_non_existing() {
        let store = get_store();

        store.delete_module_config(&module_id()).unwrap();
    }

}
