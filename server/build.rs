fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("built protos");
    tonic_build::compile_protos("../protos/goblin.proto")?;
    tonic_build::compile_protos("../protos/game.proto")?;
    Ok(())
}