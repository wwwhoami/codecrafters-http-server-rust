use anyhow::anyhow;
use std::{env, fs};

pub struct Config {
    pub port: u16,
    pub pub_dir: String,
}

impl Config {
    pub fn new(mut args: impl Iterator<Item = String>) -> crate::Result<Self> {
        let mut port = Self::parse_port_from_env()?;
        let mut pub_dir = format!("{}/public", env::current_dir().unwrap().to_string_lossy());

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-p" | "--port" => {
                    port = Self::match_port(args.next())?;
                }
                "--directory" => {
                    pub_dir = Self::match_dir(args.next())?;
                }

                _ => {}
            }
        }

        Ok(Self { port, pub_dir })
    }

    fn match_port(port_arg: Option<String>) -> crate::Result<u16> {
        let port = port_arg.ok_or(anyhow!("Port value not found"))?;

        port.parse::<u16>().map_err(|_| anyhow!("Invalid PORT"))
    }

    fn parse_port_from_env() -> crate::Result<u16> {
        // If the HTTP_PORT environment variable is not set, use the default value of 4221
        let port_str = env::var("HTTP_PORT").unwrap_or("4221".to_string());

        port_str.parse::<u16>().map_err(|_| anyhow!("Invalid PORT"))
    }

    fn match_dir(dir: Option<String>) -> crate::Result<String> {
        let path = dir.as_deref().unwrap_or("public");

        // If the directory does not exist, return an error
        fs::canonicalize(path)
            .map(|path| path.to_string_lossy().to_string())
            .map_err(|_| anyhow!("Invalid directory"))
    }
}
