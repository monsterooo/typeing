use clap::Parser;
use typeing::TypeingError;
use typeing::config::TypeingConfig;
use typeing::Typeing;

fn main() -> Result<(), TypeingError> {
    let config = TypeingConfig::parse();
    let mut typeing = Typeing::new(config)?;
    
    Ok(())
}
