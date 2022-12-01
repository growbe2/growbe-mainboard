use rusqlite::{params, Connection, Result};
use std::sync::{Arc, Mutex};

use crate::mainboardstate::error::MainboardError;

fn lock_conn(conn: &Arc<Mutex<Connection>>) -> Result<std::sync::MutexGuard<'_, rusqlite::Connection>, MainboardError> {
    return conn.try_lock().map_err(|err| MainboardError::from_error(err.to_string()));
}

pub fn get_field_from_table<T>(
    conn: &Arc<Mutex<Connection>>,
    table_name: &'static str,
    id: &String,
    id2: for<'r> fn(&'r [u8]) -> std::result::Result<T, protobuf::ProtobufError>,
) -> Result<T, MainboardError> {
    let v: Vec<u8> = lock_conn(&conn)?.query_row(
        (format!("SELECT config FROM {} WHERE id = ?", table_name)).as_str(),
        [id],
        |r| r.get(0),
    )?;
    return id2(&v).map_err(|x| MainboardError::from_protobuf_err(x));
}

pub fn get_fields_from_table<T, D>(
    conn: &Arc<Mutex<Connection>>,
    table_name: &'static str,
    property: &'static str,
    property2: &'static str,
    id2: for<'r> fn(&'r [u8]) -> std::result::Result<T, protobuf::ProtobufError>,
    id3: for<'r> fn(&'r [u8]) -> std::result::Result<D, protobuf::ProtobufError>,
) -> Result<Vec<(T, Option<D>)>, MainboardError> {
    let lock_conn = lock_conn(&conn)?;
    let mut statement = lock_conn
        .prepare((format!("SELECT {}, {} FROM {}", property, property2, table_name)).as_str())
        .map_err(|err| MainboardError::from_sqlite_err(err))?;

    return statement
        .query_map([], |row| {
            let buffer_p1: Vec<u8> = row.get(0)?;
            let buffer_p2_result: Result<Vec<u8>, rusqlite::Error> = row.get(1);

            let option_d = if let Ok(buffer_p2) = buffer_p2_result {
                if buffer_p2.len() > 0 {
                    if let Ok(v) = id3(&buffer_p2) {
                        Some(v)
                    } else {
                        log::error!("failed to parse id3 {}", property2);
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            if let Ok(v) = id2(&buffer_p1) {
                Ok((v, option_d))
            } else {
                Err(rusqlite::Error::InvalidColumnName("failed to parse id2 column".to_string()))
            }
        })?
        .map(|x| x.map_err(|x| MainboardError::from_sqlite_err(x)))
        .collect();
}

pub fn store_field_from_table(
    conn: &Arc<Mutex<Connection>>,
    table_name: &'static str,
    id: &String,
    property: &'static str,
    data: Box<dyn protobuf::Message>,
) -> Result<(), MainboardError> {
    let payload = data.write_to_bytes()?;
    let update = conn
        .lock()
        .unwrap()
        .execute(
            (format!("UPDATE {} SET {} = ? WHERE id = ?", table_name, property)).as_str(),
            params![payload, id],
        )
        .unwrap();

    if update == 0 {
        conn.lock()
            .unwrap()
            .execute(
                (format!("INSERT INTO {} (id, {}) VALUES(?,?)", table_name, property)).as_str(),
                params![id, payload],
            )
            .unwrap();
    }
    Ok(())
}

pub fn store_field_from_table_combine_key(
    conn: &Arc<Mutex<Connection>>,
    table_name: &'static str,
    id: &String,
    property: &String,
    payload: Vec<u8>,
) -> () {
    let update = conn
        .lock()
        .unwrap()
        .execute(
            (format!(
                "UPDATE {} SET config = ? WHERE id = ? AND property = ?",
                table_name
            ))
            .as_str(),
            params![payload, id, property],
        )
        .unwrap();

    if update == 0 {
        conn.lock()
            .unwrap()
            .execute(
                (format!(
                    "INSERT INTO {} (id, property, config) VALUES(?,?,?)",
                    table_name
                ))
                .as_str(),
                params![id, property, payload],
            )
            .unwrap();
    }
}

pub fn store_update_property(
    conn: &Arc<Mutex<Connection>>,
    table_name: &'static str,
    property: &'static str,
    id: &str,
    data: Box<dyn protobuf::Message>,
) -> () {
    let buffer = data.write_to_bytes().unwrap();
    conn.lock()
        .unwrap()
        .execute(
            (format!("UPDATE {} SET {} = ? WHERE id = ?", table_name, property)).as_str(),
            params![&buffer, id],
        )
        .unwrap();
}

pub fn store_update_property_combine_key(
    conn: &Arc<Mutex<Connection>>,
    table_name: &'static str,
    property: &'static str,
    id: &str,
    module_id: &str,
    payload: Vec<u8>,
) -> () {
    conn.lock()
        .unwrap()
        .execute(
            (format!(
                "UPDATE {} SET {} = ? WHERE id = ? AND property = ?",
                table_name, property
            ))
            .as_str(),
            params![payload, id, module_id],
        )
        .unwrap();
}

pub fn store_delete_key(conn: &Arc<Mutex<Connection>>, table_name: &'static str, id: &str) -> () {
    conn.lock()
        .unwrap()
        .execute(
            (format!("DELETE FROM {} WHERE id = ?", table_name)).as_str(),
            params![id],
        )
        .unwrap();
}

pub fn store_delete_combine_key(
    conn: &Arc<Mutex<Connection>>,
    table_name: &'static str,
    id: &String,
    property: &String,
) -> () {
    conn.lock()
        .unwrap()
        .execute(
            (format!("DELETE FROM {} WHERE id = ? AND property = ?", table_name)).as_str(),
            params![id, property],
        )
        .unwrap();
}

#[allow(dead_code)]
pub fn nbr_entry(conn: &Arc<Mutex<Connection>>, table: &'static str) -> i32 {
    conn.lock()
        .unwrap()
        .query_row(
            format!("SELECT count(*) FROM {}", table).as_str(),
            params![],
            |r| r.get(0),
        )
        .unwrap()
}

pub fn init() -> Connection {
    let conn = Connection::open("./database.sqlite").unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS module_config (
			id	TEXT PRIMARY KEY,
			config BLOB
		)",
        [],
    )
    .unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS module_field_alarm (
			id	TEXT,
			property TEXT,
			config BLOB,

			PRIMARY KEY (id, property)
		)",
        [],
    )
    .unwrap();

    conn.execute(
        "ALTER TABLE module_field_alarm ADD COLUMN state BLOB DEFAULT null",
        [],
    )
    .unwrap_or_default();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS virtual_relay (
			id	TEXT PRIMARY KEY,
			relay BLOB,
			config BLOB
		)",
        [],
    )
    .unwrap();

    return conn;
}
