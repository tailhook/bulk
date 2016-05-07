use config::{Config, Metadata};


pub fn populate(config: &Config) -> Result<Metadata, String> {
    let mut meta = config.metadata.clone();
    if meta.version.starts_with("v") {
        meta.version.remove(0);
    }
    Ok(meta)
}
