
pub fn get_config_path() -> String {
	let args: Vec<String> = std::env::args().collect();
	// Last argument is always config if it's a file
	let index = args.len() - 1;
    if args.len() == 0 ||  args[index].is_empty() {
        panic!("config not passed as last argument");
    }
	return args[index].clone()
}