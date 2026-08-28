fn main() {
    prost_build::compile_protos(&["../proto_src/scraper.proto"], &["../proto_src/"]).unwrap();
}
