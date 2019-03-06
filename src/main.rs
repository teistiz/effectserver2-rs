pub mod client;
pub mod config;
pub mod host;
pub mod mapper;
pub mod parser;
pub mod server;

fn main() -> std::io::Result<()> {
    let config_root = config::read_config_yaml("./config.yaml")?;

    // println!("{}", serde_yaml::to_string(&config_root).unwrap());

    let cmd_mapper = mapper::Mapper::from_config(&config_root)
        .map_err(|_| std::io::Error::from(std::io::ErrorKind::Other))?;

    server::serve(config_root, cmd_mapper)?;

    Ok(())
}
