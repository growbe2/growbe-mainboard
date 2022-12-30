extern crate protoc_rust;

use std::path::Path;

fn main() {
    let path_proto = Path::new("./proto/module.proto");
    if path_proto.exists() {
        protoc_rust::Codegen::new()
            .out_dir("./src/protos")
            .inputs(&[
                "proto/alarm.proto",
                "proto/board.proto",
                "proto/message.proto",
                "proto/module.proto",
                "proto/virt.proto",
                "proto/env_controller.proto"
            ])
            .include("proto")
            .run()
            .expect("Running protoc failed.");
    }
    #[cfg(feature = "com_i2c")]
    {
        use std::process::Command;

        Command::new("./scripts/rust_env.sh")
            .arg("make")
            .arg("-C")
            .arg("./drivers")
            .output()
            .expect("Failed to compile C");

        println!("cargo:rustc-link-search=./drivers");
    }
}
