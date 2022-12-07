use std::sync::{Arc, Mutex};

use tokio_util::sync::CancellationToken;

use crate::mainboardstate::error::MainboardError;

use super::virtual_relay::VirtualRelay;
use protobuf::Message;

pub struct VirtualRelayStore {
    pub conn: Arc<Mutex<rusqlite::Connection>>,
    pub virtual_relay_maps: std::collections::HashMap<String, VirtualRelay>,
    pub cancellation_token_maps: std::collections::HashMap<String, CancellationToken>,
}

/*
 * VirtualRelayStore is the object in charge of saving the state and config
 * of the virtual relay
 */
impl VirtualRelayStore {
    pub fn new(conn: Arc<Mutex<rusqlite::Connection>>) -> Self {
        VirtualRelayStore {
            conn,
            virtual_relay_maps: std::collections::HashMap::new(),
            cancellation_token_maps: std::collections::HashMap::new(),
        }
    }

    /*
     * is_created tell if a virtual relay is existing and if it's runnning
     */
    pub fn is_created(&self, virtual_relay_id: &str) -> bool {
        return self.virtual_relay_maps.contains_key(virtual_relay_id);
    }

    /*
     * stop_virtual_relay stop the virtual relay but keep the config store to allow to start it again.
     * cancel all attach running task
     * (TODO) support the virtual relay to go back to a default state (all close)
     */
    pub fn stop_virtual_relay(&mut self, id: &str) {
        let d = self.virtual_relay_maps.remove(id);
        if d.is_some() {
            if let Some(cancellation_token) = self.cancellation_token_maps.remove(id) {
                cancellation_token.cancel();
            }
        }
    }

    /*
     * store_relay store the relay definition in the local store, throw error if already existings
     */
    pub fn store_relay(
        &self,
        config: &crate::protos::module::VirtualRelay,
    ) -> Result<(), MainboardError> {
        return crate::store::database::store_field_from_table(
            &self.conn,
            "virtual_relay",
            &String::from(config.get_name()),
            "relay",
            Box::new(config.clone()),
        );
    }

    /*
     * remove_relay remove an existing virtual relay definition.
     * stopping if existing
     * removing from database
     */
    pub fn remove_relay(&mut self, id: &str) -> Result<(), MainboardError> {
        self.stop_virtual_relay(id);
        crate::store::database::store_delete_key(&self.conn, "virtual_relay", id)?;
        Ok(())
    }

    /*
     * get_stores_relays return the list of virtual_relay existing in the database
     */
    pub fn get_stored_relays(
        &self,
    ) -> Result<
        Vec<(
            crate::protos::module::VirtualRelay,
            Option<crate::protos::module::RelayOutletConfig>,
        )>,
        MainboardError,
    > {
        return crate::store::database::get_fields_from_table(
            &self.conn,
            "virtual_relay",
            "relay",
            "config",
            crate::protos::module::VirtualRelay::parse_from_bytes,
            crate::protos::module::RelayOutletConfig::parse_from_bytes,
        );
    }

    /*
     * store_relay_config , store the config of a relay
     */
    pub fn store_relay_config(
        &self,
        id: &str,
        config: &crate::protos::module::RelayOutletConfig,
    ) -> Result<(), MainboardError> {
        crate::store::database::store_update_property(
            &self.conn,
            "virtual_relay",
            "config",
            id,
            Box::new(config.clone()),
        )?;
        return Ok(());
    }
}
