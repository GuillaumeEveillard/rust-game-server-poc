fn main() {
    tonic_build::compile_protos("proto/gamemaster.proto").unwrap();
}
