pub fn last_element_path(path: &str) -> Option<String> {
    let topic_elements: Vec<&str> = path.split("/").collect();
    return match topic_elements.len() {
        0 => None,
        n => Some(String::from(topic_elements[n - 1])),
    };
}
