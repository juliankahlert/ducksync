use clap::Parser;
use directories::BaseDirs;
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

use std::collections::HashMap;

pub struct DuckDns;

impl DuckDns {
    // Update the DuckDNS record with the provided parameters.
    // This will send a request to DuckDNS API to update the IP address for the domains.
    pub async fn update(
        domains: Vec<String>,
        token: String,
        ip: Option<String>,
        ipv6: Option<String>,
        verbose: Option<bool>,
        clear: Option<bool>,
    ) -> Result<(), String> {
        let mut params = HashMap::new();
        params.insert("domains", domains.join(","));
        params.insert("token", token);

        if let Some(ip_value) = ip {
            params.insert("ip", ip_value);
        }

        if let Some(ipv6_value) = ipv6 {
            params.insert("ipv6", ipv6_value);
        }

        if let Some(verbose_value) = verbose {
            params.insert("verbose", verbose_value.to_string());
        }

        if let Some(clear_value) = clear {
            params.insert("clear", clear_value.to_string());
        }

        let url = "https://www.duckdns.org/update";

        let client = Client::new();
        let response = client
            .get(url)
            .query(&params)
            .send()
            .await
            .map_err(|e| format!("Error sending request: {}", e))?;

        let body = response
            .text()
            .await
            .map_err(|e| format!("Error reading response body: {}", e))?;

        if body.starts_with("OK") {
            Ok(())
        } else {
            Err(format!("Update failed: {}", body))
        }
    }

    // Update the DuckDNS TXT record with the provided parameters.
    // This will send a request to DuckDNS API to update the TXT record for the domains.
    pub async fn update_txt(
        domains: Vec<String>,
        token: String,
        txt: String,
        verbose: Option<bool>,
        clear: Option<bool>,
    ) -> Result<(), String> {
        let mut params = HashMap::new();
        params.insert("domains", domains.join(","));
        params.insert("token", token);
        params.insert("txt", txt);

        if let Some(verbose_value) = verbose {
            params.insert("verbose", verbose_value.to_string());
        }

        if let Some(clear_value) = clear {
            params.insert("clear", clear_value.to_string());
        }

        let url = "https://www.duckdns.org/update";

        let client = Client::new();
        let response = client
            .get(url)
            .query(&params)
            .send()
            .await
            .map_err(|e| format!("Error sending request: {}", e))?;

        let body = response
            .text()
            .await
            .map_err(|e| format!("Error reading response body: {}", e))?;

        if body.starts_with("OK") {
            Ok(())
        } else {
            Err(format!("Update failed: {}", body))
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

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
                    let res = DuckDns::update(
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
