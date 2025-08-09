fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::compile_protos("./todo-service-proto/v1/todo.proto")?;

    Ok(())
}
