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

lazy_static::lazy_static! {
    static ref SUPPORTED_MODULES: Vec<&'static str> = vec!["AAP", "AAB", "AAS", "PCS", "CCS"];
}

fn is_supported(module_id: &String) -> bool {
    if module_id.len() > 3 {
        let sub = module_id[0..3].to_string();
        for sup in SUPPORTED_MODULES.iter() {
            if sub.eq(sup) {
                return true;
            }
        }
    }
    return false;
}

impl ModuleStateStore {
    pub fn new(conn: Arc<Mutex<rusqlite::Connection>>) -> Self {
        return ModuleStateStore { conn };
    }

    pub fn get_module_config(&self, id: &String) -> Option<(Box<dyn Message>, bool)> {
        let module_type = &id[..3];
        let item = match module_type {
            "AAP" => {
                let result = self.get_module_config_inner(id, RelayModuleConfig::parse_from_bytes);
                if let Ok(d) = result {
                    Ok((Box::new(d) as Box<dyn Message>, false))
                } else {
                    log::info!("failed to get module config generating a new one");
                    Ok((Box::new(RelayModuleConfig::new()) as Box<dyn Message>, true))
                }
            }
            "AAB" => {
                let result = self.get_module_config_inner(id, WCModuleConfig::parse_from_bytes);
                if let Ok(d) = result {
                    Ok((Box::new(d) as Box<dyn Message>, false))
                } else {
                    log::info!("failed to get module config generating a new one");
                    Ok((Box::new(WCModuleConfig::new()) as Box<dyn Message>, true))
                }
            }
            "AAS" => {
                let result = self.get_module_config_inner(id, SOILModuleConfig::parse_from_bytes);
                if let Ok(config) = result {
                    Ok((Box::new(config) as Box<dyn Message>, false))
                } else {
                    Err(super::interface::ModuleError::new())
                }
            }
            "PCS" => {
                let result =
                    self.get_module_config_inner(id, PhoneStreamingConfig::parse_from_bytes);
                if let Ok(config) = result {
                    Ok((Box::new(config) as Box<dyn Message>, false))
                } else {
                    Err(super::interface::ModuleError::new())
                }
            }
            "CCS" => {
                let result =
                    self.get_module_config_inner(id, ComputerStreamingConfig::parse_from_bytes);
                if let Ok(config) = result {
                    Ok((Box::new(config) as Box<dyn Message>, false))
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

    pub fn store_module_config(
        &self,
        id: &String,
        config: &Box<dyn protobuf::Message>,
    ) -> Result<(), MainboardError> {
        if is_supported(id) {
            log::debug!("store module config {}", id);
            return database::store_field_from_table(
                &self.conn,
                "module_config",
                id,
                "config",
                config,
            );
        }
        return Err(MainboardError::from_error(
            "module is not supported to save config".to_string(),
        ));
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        mainboardstate::hello_world, protos::board::HelloWord, store::database::nbr_entry,
    };
    use std::sync::{Arc, Mutex};

    fn supported_modules() -> Vec<&'static str> {
        return vec!["AAP", "AAB", "AAS", "PCS", "CCS"];
    }

    fn get_id() -> &'static str {
        "0000001"
    }

    fn get_modules() -> Vec<String> {
        supported_modules()
            .into_iter()
            .map(|x| format!("{}{}", x, get_id()))
            .collect()
    }

    fn module_id() -> String {
        "AAP0000003".to_string()
    }

    fn get_store() -> ModuleStateStore {
        let conn_database = Arc::new(Mutex::new(crate::store::database::init(Some(
            "./database_test_modulestate.sqlite".to_string(),
        ))));
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
    fn store_module_config_not_existing_create_new_one() {
        let store = get_store();

        let config_opt = store.get_module_config(&module_id());

        let config = config_opt.unwrap();

        assert_eq!(config.1, true);
    }

    #[test]
    fn store_module_config_existing() {
        let store = get_store();

        let config: Box<dyn Message> = Box::new(RelayModuleConfig::new());
        store.store_module_config(&module_id(), &config).unwrap();

        store.get_module_config(&module_id()).unwrap();

        assert_eq!(nbr_entry(&store.conn, "module_config").unwrap(), 1);
    }

    #[test]
    fn store_module_deleting_non_existing() {
        let store = get_store();

        store.delete_module_config(&module_id()).unwrap();
    }

    #[test]
    fn store_all_supported_type_are_working() {
        let store = get_store();

        for module in get_modules() {
            let config: Box<dyn protobuf::Message> = Box::new(SOILModuleConfig::new());
            store.store_module_config(&module, &config).unwrap();
            store.get_module_config(&module).unwrap();
        }
    }

    #[test]
    fn store_unsupported_module_type() {
        let store = get_store();
        let config: Box<dyn protobuf::Message> = Box::new(SOILModuleConfig::new());
        let id = "KKK0000001".to_string();

        store.store_module_config(&id, &config).unwrap_err();

        assert_eq!(store.get_module_config(&id).is_none(), true);
    }
}
