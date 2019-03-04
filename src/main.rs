use std::collections::HashMap;

mod config;
mod host;
mod mapper;
mod server;

fn main() -> std::io::Result<()> {
    let config_root = config::read_config_json("./config.json")?;

    println!("{:?}", config_root);

    let cmd_mapper = mapper::Mapper::from_config(&config_root)
        .map_err(|_| { std::io::Error::from(std::io::ErrorKind::Other)})?;

    let server = server::serve(config_root, cmd_mapper);

    println!("Hello, world!");

    Ok(())
}
