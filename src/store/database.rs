use rusqlite::{params, Connection, Result, ToSql, Error};
use rusqlite::types::{FromSqlResult, ToSqlOutput, ValueRef, FromSqlError, FromSql};


use protobuf::Message;
use crate::modulestate::interface::ModuleValueParsable;


impl ToSql for dyn ModuleValueParsable {
	#[inline]
    fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.write_to_bytes().unwrap()))
    }
}

/*impl FromSql for dyn ModuleValueParsable {
    #[inline]
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Box<Self>> {
        let bytes = value.as_bytes().unwrap();
		let data = Box::new(crate::protos::module::SOILModuleData::parse_from_bytes(&bytes).unwrap());
		Ok(data)
    }
}*/

pub fn init() -> Connection {
	let conn = Connection::open("./database.sqlite").unwrap();

	conn.execute(
		"CREATE TABLE IF NOT EXISTS module_config (
			id	TEXT PRIMARY KEY,
			config BLOB
		)",
		[]
	).unwrap();

	/*
	let mut data = crate::protos::module::SOILModuleData::new();
	data.p0 = 50;

	/*conn.execute(
		"INSERT INTO module_config (id, config) VALUES ('AAAP',?)",
		&[&data as &dyn ModuleValueParsable]
	).unwrap();*/

	let v: Vec<u8> = conn.query_row("SELECT config FROM module_config", [],
	|r| r.get(0),
	).unwrap();

	let soil_data = crate::protos::module::SOILModuleData::parse_from_bytes(&v).unwrap();
	println!("Read back butes {:?}", soil_data);
	*/

	return conn;
}