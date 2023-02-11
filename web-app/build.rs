use std::{
    env,
    fs::{self, File},
    io::Read,
    path::Path,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
struct AppConfig {
    endpoints: Vec<String>,
    pixel_count: usize,
    initial_js_file: String,
    initial_js: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_source = config::Config::builder()
        // Add in `./Settings.toml`
        .add_source(config::File::with_name("config"))
        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        .add_source(config::Environment::with_prefix("TRENCHCOAT"))
        .build()?;

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("config.ser");

    let mut config: AppConfig = config_source.try_deserialize()?;
    let mut initial_js = File::open(&config.initial_js_file)?;
    let mut js_contents = String::new();
    initial_js.read_to_string(&mut js_contents)?;
    config.initial_js = Some(js_contents);

    let config_ser = postcard::to_allocvec(&config)?;
    fs::write(&dest_path, config_ser)?;
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../res");
    Ok(())
}
