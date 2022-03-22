use crate::protos::alarm::FieldAlarm;
use crate::modulestate::interface::ModuleError;
use crate::store::database::{store_field_from_table_combine_key, store_delete_combine_key, store_update_property_combine_key};

use protobuf::Message;


use std::sync::{Arc, Mutex};

pub struct ModuleAlarmStore {
    pub conn: Arc<Mutex<rusqlite::Connection>>,
}

// ModuleAlarmStore is the store where we keep our alarm field config
impl ModuleAlarmStore {
        
    pub fn new(conn: Arc<Mutex<rusqlite::Connection>>) -> Self {
        return ModuleAlarmStore{
            conn,
        };
    }

    pub fn get_alarm_for_module(&self, module_id: &String) -> Result<Vec<FieldAlarm>, ModuleError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT config FROM module_field_alarm WHERE id = ?").unwrap();
        let vec_alarms = stmt.query_map([module_id.as_str()], |row| {
            let buffer: Vec<u8> = row.get(0).unwrap();
            Ok(FieldAlarm::parse_from_bytes(&buffer).unwrap())
        }).unwrap().map(|i| i.unwrap()).collect();
        Ok(vec_alarms)
    }

    pub fn add_alarm_field(&self, alarm: &FieldAlarm) -> Result<(), ModuleError>  {
        store_field_from_table_combine_key(&self.conn, "module_field_alarm", &alarm.moduleId.clone(), &alarm.property.clone(), alarm.write_to_bytes().unwrap());
        Ok(())
    }

    pub fn update_alarm_field(&self, alarm: &FieldAlarm) -> Result<(), ModuleError> {
        store_update_property_combine_key(&self.conn, "module_field_alarm", "config", &alarm.moduleId.clone(), &alarm.property.clone(), alarm.write_to_bytes().unwrap());
        Ok(())
    }

    pub fn remove_alarm_field(&self, alarm: &FieldAlarm) -> Result<(), ModuleError> {
        store_delete_combine_key(&self.conn, "module_field_alarm", &alarm.moduleId, &alarm.property);
        Ok(())
    }
}



#[cfg(test)]
mod tests {

    use super::*;
    use rusqlite::{params};
    use std::sync::{Mutex, Arc};
    use crate::store::database::nbr_entry;
    use crate::protos::alarm::FieldAlarm;
    

    const MODULE_ID: &str = "AAA0000003";
    const PROPERTY: &str = "p0";
    const PROPERTY_2: &str = "p1";

    fn get_store() -> ModuleAlarmStore {
        let conn_database = Arc::new(Mutex::new(crate::store::database::init()));
        let store = ModuleAlarmStore::new(conn_database);
        clear_store(&store);
        store
    }

    fn clear_store(store: &ModuleAlarmStore) {
        store.conn.lock().unwrap().execute(
            "DELETE FROM module_field_alarm",
            params![]
        ).unwrap();
    }

    #[test] 
    fn store_alarm_field_not_existing() {
        let store = get_store();

        let mut field_alarm = FieldAlarm::new();
        field_alarm.moduleId = MODULE_ID.to_string();
        field_alarm.property = PROPERTY.to_string();

        store.add_alarm_field(&field_alarm).unwrap();

        assert_eq!(nbr_entry(&store.conn, "module_field_alarm"), 1);

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

        assert_eq!(nbr_entry(&store.conn, "module_field_alarm"), 1);

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

        assert_eq!(nbr_entry(&store.conn, "module_field_alarm"), 0);

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
