extern crate protoc_rust;

use std::path::Path;

fn main() {
    let path_proto = Path::new("./proto/module.proto");
    if path_proto.exists() {
        let mut customize = protoc_rust::Customize::default();
        customize.serde_derive = Some(true);
        customize.serde_derive_cfg = Some("".to_string());
        protoc_rust::Codegen::new()
            .out_dir("./src/protos")
            .customize(customize)
            .inputs(&[
                "proto/alarm.proto",
                "proto/board.proto",
                "proto/message.proto",
                "proto/sync.proto",
                "proto/module.proto",
                "proto/virt.proto",
                "proto/env_controller.proto",
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
