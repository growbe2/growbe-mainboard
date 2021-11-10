extern crate protoc_rust;

use std::process::Command;


fn main() {
    // uncomment to generate protobuf , need to be copy to from growbe-cloud
    /*protoc_rust::Codegen::new()
        .out_dir("./src/protos")
        .inputs(&["./proto/alarm.proto", "proto/board.proto", "proto/message.proto", "proto/module.proto"])
        .include("proto")
        .run()
        .expect("Running protoc failed.");*/

        Command::new("./scripts/rust_env.sh")
        .arg("make").arg("-C").arg("./drivers")
        .output()
        .expect("Failed to compile C");


        println!("cargo:rustc-link-search=./drivers");
}