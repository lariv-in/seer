use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(&["../proto_src/scraper.proto"], &["../proto_src/"])?;
    Ok(())
}
