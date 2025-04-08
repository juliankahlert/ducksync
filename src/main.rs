use clap::Parser;
use ducksync::config::{Config, Ip};
use ducksync::duckdns::DuckDns;
use reqwest::Client;
use std::net::IpAddr;

#[derive(Parser, Debug)]
#[command(name = "ducksync")]
struct Args {
    #[arg(short, long)]
    config: Option<String>,
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

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let duckdns = DuckDns::new();

    match Config::load(args.config).await {
        Ok(config) => {
            println!("Successfully loaded config:");

            for domain in config.domains {
                println!("{:?}", domain);

                if let Some(Ip::Public) = domain.ip {
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
