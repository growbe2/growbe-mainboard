pub fn handle_command_line_arguments() -> Option<()> {
    if let Some(first_arg) = std::env::args().skip(1).next() {
        match first_arg.as_str() {
            "version" => {
                println!("{}", crate::version::VERSION.to_string());
                return Some(());
            }
            "id" => {
                println!("{}", crate::id::get());
                return Some(());
            }
            _ => {}
        }
    }

    return None;
}
