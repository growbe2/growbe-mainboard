pub fn last_element_path(path: &str) -> Option<String> {
    let topic_elements: Vec<&str> = path.split("/").collect();
    return match topic_elements.len() {
        0 => None,
        n => Some(String::from(topic_elements[n - 1])),
    };
}

pub fn extract_module_id(topic_name: &String) -> String {
    let pieces: Vec<&str> = topic_name.split("/").collect();
    let last = pieces.get(pieces.len() - 1).unwrap();
    return String::from(last.clone());
}
