fn main() {
    let proto_files = vec!["../service/proto/api.proto"];

    protoc_rust::Codegen::new()
        .out_dir("./src")
        .inputs(proto_files)
        .include("../service/proto")
        .run()
        .expect("running protoc failed");
}
