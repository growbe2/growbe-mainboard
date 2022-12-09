use crate::mainboardstate::error::MainboardError;
use crate::modulestate::interface::{conv_err, ModuleError};
use crate::protos::alarm::FieldAlarm;
use crate::store::database::{
    store_delete_combine_key, store_field_from_table_combine_key,
    store_update_property_combine_key, to_sqerror,
};

use protobuf::Message;
use rusqlite::Row;

use std::sync::{Arc, Mutex};

use super::model::ModuleAlarmState;

pub struct ModuleAlarmStore {
    pub conn: Arc<Mutex<rusqlite::Connection>>,
}

fn handle_row(row: &Row) -> Result<(FieldAlarm, Option<ModuleAlarmState<f32>>), rusqlite::Error> {
    let buffer: Vec<u8> = row.get(0)?;
    let buffer_state: Option<Vec<u8>> = row.get(1)?;
    let buffer_state = buffer_state.unwrap_or_default();
    let state = if buffer_state.len() > 0 {
        Some(serde_json::from_slice(&buffer_state).map_err(to_sqerror)?)
    } else {
        None
    };
    Ok((
        FieldAlarm::parse_from_bytes(&buffer).map_err(to_sqerror)?,
        state,
    ))
}

// ModuleAlarmStore is the store where we keep our alarm field config
impl ModuleAlarmStore {
    pub fn new(conn: Arc<Mutex<rusqlite::Connection>>) -> Self {
        return ModuleAlarmStore { conn };
    }

    pub fn get_alarm_for_module_property(
        &self,
        module_id: &str,
        property: &str,
    ) -> Result<(FieldAlarm, Option<super::model::ModuleAlarmState<f32>>), MainboardError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT config, state FROM module_field_alarm WHERE id = ? AND property = ?")
            .unwrap();
        let dd: Result<
            Vec<(FieldAlarm, Option<super::model::ModuleAlarmState<f32>>)>,
            ModuleError,
        > = stmt
            .query_map([module_id, property], handle_row)?
            .map(|x| x.map_err(|x| ModuleError::from_rusqlite_err(module_id, x)))
            .collect();

        let mut dd = dd?;

        if let Some(value) = dd.pop() {
            return Ok(value);
        } else {
            return Err(MainboardError::from_error(format!(
                "alarm not found [{}:{}]",
                module_id, property
            )));
        }
    }

    pub fn get_alarm_for_module(
        &self,
        module_id: &String,
    ) -> Result<Vec<(FieldAlarm, Option<super::model::ModuleAlarmState<f32>>)>, ModuleError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT config, state FROM module_field_alarm WHERE id = ?")
            
            .unwrap();
        return stmt
            .query_map([module_id.as_str()], handle_row)?
            .map(|x| x.map_err(|x| ModuleError::from_rusqlite_err(module_id, x)))
            .collect();
    }

    pub fn add_alarm_field(&self, alarm: &FieldAlarm) -> Result<(), ModuleError> {
        store_field_from_table_combine_key(
            &self.conn,
            "module_field_alarm",
            &alarm.moduleId.clone(),
            &alarm.property.clone(),
            alarm
                .write_to_bytes()
                .map_err(|x| ModuleError::from_protobuf_err(&alarm.moduleId, x))?,
        )
        .map_err(conv_err(alarm.moduleId.clone()))?;
        Ok(())
    }

    pub fn update_alarm_state(
        &self,
        module_id: &str,
        property: &str,
        alarm_state: &super::model::ModuleAlarmState<f32>,
    ) -> Result<(), ModuleError> {
        let payload = serde_json::to_vec(&alarm_state).map_err(|x| {
            return ModuleError::new()
                .module_id(module_id.to_string())
                .message(x.to_string());
        })?;
        store_update_property_combine_key(
            &self.conn,
            "module_field_alarm",
            "state",
            module_id,
            property,
            payload,
        )
        .map_err(conv_err(module_id.to_string()))?;
        Ok(())
    }

    pub fn update_alarm_field(&self, alarm: &FieldAlarm) -> Result<(), ModuleError> {
        store_update_property_combine_key(
            &self.conn,
            "module_field_alarm",
            "config",
            &alarm.moduleId.clone(),
            &alarm.property.clone(),
            alarm
                .write_to_bytes()
                .map_err(|x| ModuleError::from_protobuf_err(&alarm.moduleId, x))?,
        )
        .map_err(conv_err(alarm.moduleId.clone()))?;
        Ok(())
    }

    pub fn remove_alarm_field(&self, alarm: &FieldAlarm) -> Result<(), ModuleError> {
        store_delete_combine_key(
            &self.conn,
            "module_field_alarm",
            &alarm.moduleId,
            &alarm.property,
        )
        .map_err(conv_err(alarm.moduleId.clone()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::protos::alarm::FieldAlarm;
    use crate::store::database::nbr_entry;
    use std::sync::{Arc, Mutex};

    const MODULE_ID: &str = "AAA0000003";
    const PROPERTY: &str = "p0";
    const PROPERTY_2: &str = "p1";

    fn get_store() -> ModuleAlarmStore {
        let conn_database = Arc::new(Mutex::new(crate::store::database::init(Some(
            "./database_test_alarm.sqlite".to_string(),
        ))));
        let store = ModuleAlarmStore::new(conn_database);
        clear_store(&store);
        store
    }

    fn clear_store(store: &ModuleAlarmStore) {
        store
            .conn
            .lock()
            .unwrap()
            .execute("DELETE FROM module_field_alarm", [])
            .unwrap();
    }

    #[test]
    fn store_alarm_field_not_existing() {
        let store = get_store();

        let mut field_alarm = FieldAlarm::new();
        field_alarm.moduleId = MODULE_ID.to_string();
        field_alarm.property = PROPERTY.to_string();

        store.add_alarm_field(&field_alarm).unwrap();

        assert_eq!(nbr_entry(&store.conn, "module_field_alarm").unwrap(), 1);

        clear_store(&store);
    }

    #[test]
    fn store_alarm_fiend_update_state() {
        let store = get_store();

        let mut field_alarm = FieldAlarm::new();
        field_alarm.moduleId = MODULE_ID.to_string();
        field_alarm.property = PROPERTY.to_string();

        store.add_alarm_field(&field_alarm).unwrap();

        let mut state = crate::modulestate::alarm::model::ModuleAlarmState::<f32>::default();
        state.current_value = 5.;

        store
            .update_alarm_state(
                field_alarm.moduleId.as_str(),
                field_alarm.property.as_str(),
                &state,
            )
            .unwrap();

        let alarms = store.get_alarm_for_module(&field_alarm.moduleId).unwrap();

        let (_field_alarm, state) = alarms.get(0).unwrap();

        let state = state.as_ref().unwrap();

        assert_eq!(state.current_value, 5.);

        clear_store(&store);
    }

    #[test]
    fn store_alarm_field_already_existing() {
        let store = get_store();

        let mut field_alarm = FieldAlarm::new();
        field_alarm.moduleId = MODULE_ID.to_string();
        field_alarm.property = PROPERTY.to_string();

        store.add_alarm_field(&field_alarm).unwrap();
        store.add_alarm_field(&field_alarm).unwrap();

        assert_eq!(nbr_entry(&store.conn, "module_field_alarm").unwrap(), 1);

        clear_store(&store);
    }

    #[test]
    fn store_alarm_field_and_remove_it() {
        let store = get_store();

        let mut field_alarm = FieldAlarm::new();
        field_alarm.moduleId = MODULE_ID.to_string();
        field_alarm.property = PROPERTY.to_string();

        store.add_alarm_field(&field_alarm).unwrap();
        store.remove_alarm_field(&field_alarm).unwrap();

        assert_eq!(nbr_entry(&store.conn, "module_field_alarm").unwrap(), 0);

        clear_store(&store);
    }

    #[test]
    fn store_alarm_field_and_get_module_alarms() {
        let store = get_store();

        let mut field_alarm = FieldAlarm::new();
        field_alarm.moduleId = MODULE_ID.to_string();
        field_alarm.property = PROPERTY.to_string();

        store.add_alarm_field(&field_alarm).unwrap();

        field_alarm.property = PROPERTY_2.to_string();

        store.add_alarm_field(&field_alarm).unwrap();

        // add to another module so should not show up
        field_alarm.moduleId = "12345".to_string();
        store.add_alarm_field(&field_alarm).unwrap();

        let alarms = store.get_alarm_for_module(&MODULE_ID.to_string()).unwrap();

        assert_eq!(alarms.len(), 2);

        clear_store(&store);
    }
}
