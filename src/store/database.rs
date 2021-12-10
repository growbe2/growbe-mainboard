use rusqlite::{params, Connection, Result};
use std::sync::{Mutex, Arc};


pub fn get_field_from_table<T>(
	conn: &Arc<Mutex<Connection>>,
	table_name: &'static str,
	id: &String,
	id2: for<'r> fn(&'r [u8]) -> std::result::Result<T, protobuf::ProtobufError>,

) -> Result<T, rusqlite::Error> {
	let v: Vec<u8> = conn.try_lock().unwrap().query_row(
		(format!("SELECT config FROM {} WHERE id = ?", table_name)).as_str(),
		[id],
		|r| r.get(0)
	)?;
	return Ok(id2(&v).unwrap());
}

pub fn store_field_from_table(
	conn: &Arc<Mutex<Connection>>,
	table_name: &'static str,
	id: &String,
	data: Box<dyn protobuf::Message>,
) -> () {
	let payload = data.write_to_bytes().unwrap();
	let update = conn.lock().unwrap().execute(
		(format!("UPDATE {} SET config = ? WHERE id = ?", table_name)).as_str(),
		params![payload, id],
	).unwrap();

	if update == 0 {
		conn.lock().unwrap().execute(
			(format!("INSERT INTO {} (id, config) VALUES(?,?)", table_name)).as_str(),
			params![id, payload]
		).unwrap();
	}
}

pub fn store_field_from_table_combine_key(
	conn: &Arc<Mutex<Connection>>,
	table_name: &'static str,
	id: &String,
	property: &String,
	payload: Vec<u8>,
) -> () {
	let update = conn.lock().unwrap().execute(
		(format!("UPDATE {} SET config = ? WHERE id = ? AND property = ?", table_name)).as_str(),
		params![payload, id, property],
	).unwrap();

	if update == 0 {
		conn.lock().unwrap().execute(
			(format!("INSERT INTO {} (id, property, config) VALUES(?,?,?)", table_name)).as_str(),
			params![id, property, payload]
		).unwrap();
	}
}

pub fn store_delete_combine_key(
	conn: &Arc<Mutex<Connection>>,
	table_name: &'static str,
	id: &String,
	property: &String,
) -> () {
	conn.lock().unwrap().execute(
		(format!("DELETE FROM {} WHERE id = ? AND property = ?", table_name)).as_str(),
		params![id, property]
	).unwrap();
}

pub fn nbr_entry(
    conn: &Arc<Mutex<Connection>>,
    table: &'static str,
) -> i32 {
	conn.lock().unwrap().query_row(
		format!("SELECT count(*) FROM {}", table).as_str(),
		params![],
		|r| r.get(0)
	).unwrap()
}


pub fn init() -> Connection {
	let conn = Connection::open("./database.sqlite").unwrap();

	conn.execute(
		"CREATE TABLE IF NOT EXISTS module_config (
			id	TEXT PRIMARY KEY,
			config BLOB
		)",
		[]
	).unwrap();

	conn.execute(
		"CREATE TABLE IF NOT EXISTS module_field_alarm (
			id	TEXT,
			property TEXT,
			config BLOB,

			PRIMARY KEY (id, property)
		)",
		[]
	).unwrap();

	return conn;
}