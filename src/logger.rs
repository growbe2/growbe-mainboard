use std::env;
use std::io::Write;
use chrono::Local;
use env_logger::Builder;


pub fn setup_log() {
	env_logger::init();
}