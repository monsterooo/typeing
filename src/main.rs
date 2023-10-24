use clap::Parser;
use typeing::TypeingError;
use typeing::config::TypeingConfig;

fn main() -> Result<(), TypeingError> {
    let config = TypeingConfig::parse();
    let mut typeing = Typeing::new(config)?;
    
    Ok(())
}
