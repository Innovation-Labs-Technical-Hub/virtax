// Proto codegen with a vendored protoc, so contributors need no system protoc install.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = prost_build::Config::new();
    config.protoc_executable(protoc_bin_vendored::protoc_bin_path()?);
    let include = protoc_bin_vendored::include_path()?;
    tonic_prost_build::configure().compile_with_config(
        config,
        &["proto/oxbox/v1/oxbox.proto".into()],
        &["proto".into(), include],
    )?;
    println!("cargo:rerun-if-changed=proto");
    Ok(())
}
