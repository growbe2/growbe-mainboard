pub fn read_serde_json_file<T>(path: String) -> T
where
    T: DeserializeOwned,
{
    let file = std::fs::File::open(path).expect("Error open file");
    return serde_json::from_reader(file).unwrap();
}

pub fn read_proto_json_file<T>(path: String) -> T
where
    T: protobuf::Message,
{
}
