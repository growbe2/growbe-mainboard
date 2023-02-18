extern crate protoc_rust;

use std::path::Path;

fn main() {
    let path_proto = Path::new("../../proto/module.proto");
    if path_proto.exists() {
        protoc_rust::Codegen::new()
            .out_dir("./src/protos")
            .inputs(&["../../proto/module.proto", "../../proto/sync.proto"])
            .include("../../proto")
            .run()
            .expect("Running protoc failed.");
    }
}
