pub fn last_element_path(path: &str) -> String {
    let topic_elements: Vec<&str> = path.split("/").collect();
    return match topic_elements.len() {
        0 => panic!("Failed to get module id from path"),
        n => String::from(topic_elements[n - 1]),
    };
}
