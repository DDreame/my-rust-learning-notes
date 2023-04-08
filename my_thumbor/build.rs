
extern crate prost_build;
fn main() {
    prost_build::Config::new()
        .out_dir("src/pb")
        .compile_protos(&["abi.proto"], &["./"])
        .unwrap();
}