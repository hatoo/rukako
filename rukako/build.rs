use std::error::Error;

use spirv_builder::{MetadataPrintout, SpirvBuilder};

fn main() -> Result<(), Box<dyn Error>> {
    SpirvBuilder::new("../rukako-shader", "spirv-unknown-spv1.3")
        .print_metadata(MetadataPrintout::Full)
        .build()?;

    Ok(())
}
