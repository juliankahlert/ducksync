use clap::Parser;
use directories::BaseDirs;
use ducksync::duckdns::DuckDns;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::net::IpAddr;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use tokio::fs;
use tokio::io::{self, ErrorKind};

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    domains: Vec<Domain>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Domain {
    name: String,
    token: String,
    ip: Option<ConfigIp>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ConfigIp {
    Public,
    IPv4(String),
    IPv6(String),
}

#[derive(Parser, Debug)]
#[command(name = "ducksync")]
struct Args {
    #[arg(short, long)]
    config: Option<String>,
}

async fn check_secure_file_mode(path: &str) -> io::Result<()> {
    let metadata = fs::metadata(path).await?;
    let permissions = metadata.permissions();
    let mode = permissions.mode() & 0o777;

    if mode != 0o600 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("Config file {} must have mode 600", path),
        ));
    }
    Ok(())
}

async fn resolve_public_ip() -> Result<IpAddr, String> {
    let client = Client::new();
    let public_ip: String = client
        .get("https://api.ipify.org?format=text")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    public_ip.parse::<IpAddr>().map_err(|e| e.to_string())
}

async fn resolve_public_ipv6() -> Result<IpAddr, String> {
    let client = reqwest::Client::new();
    let public_ip: String = client
        .get("https://api6.ipify.org?format=text")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    public_ip.parse::<IpAddr>().map_err(|e| e.to_string())
}

async fn load_config(config_path: Option<String>) -> io::Result<Config> {
    let user_config_path = match BaseDirs::new() {
        Some(dirs) => dirs
            .home_dir()
            .join(".config")
            .join("ducksync")
            .join("config.yaml"),
        None => {
            return Err(io::Error::new(
                ErrorKind::NotFound,
                "Home directory not found",
            ));
        }
    };

    let system_config_path = Path::new("/etc/ducksync/config.yaml");

    let config_path = if let Some(custom_path) = config_path {
        Path::new(&custom_path).to_path_buf()
    } else if user_config_path.exists() {
        user_config_path
    } else if system_config_path.exists() {
        system_config_path.to_path_buf()
    } else {
        return Err(io::Error::new(
            ErrorKind::NotFound,
            "Config file not found in either user or system path",
        ));
    };

    check_secure_file_mode(config_path.to_str().unwrap()).await?;

    let config_content = fs::read_to_string(config_path).await?;

    let config: Config = serde_yaml::from_str(&config_content)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

    Ok(config)
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let duckdns = DuckDns::new();

    match load_config(args.config).await {
        Ok(config) => {
            println!("Successfully loaded config:");

            for domain in config.domains {
                println!("{:?}", domain);

                if let Some(ConfigIp::Public) = domain.ip {
                    let res = match resolve_public_ipv6().await {
                        Ok(ip) => Ok(ip),
                        Err(_) => resolve_public_ip().await,
                    };

                    let Ok(ip) = res else {
                        continue;
                    };

                    println!("IP found for domain {} => {}", &domain.name, &ip);
                    let res = duckdns
                        .update(
                            vec![domain.name.clone()],
                            domain.token,
                            Some(ip.to_string()),
                            None,
                            None,
                            None,
                        )
                        .await;
                    println!("{:?}", res);
                }
            }
        }
        Err(e) => eprintln!("Error loading config: {}", e),
    }
}
