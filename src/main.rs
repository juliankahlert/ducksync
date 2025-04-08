use clap::Parser;
use ducksync::config::{Config, Ip};
use ducksync::duckdns::DuckDns;
use ducksync::ipify::Ipify;

#[derive(Parser, Debug)]
#[command(name = "ducksync")]
struct Args {
    #[arg(short, long)]
    config: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let duckdns = DuckDns::new();
    let ipify = Ipify::new();

    match Config::load(args.config).await {
        Ok(config) => {
            println!("Successfully loaded config:");

            for domain in config.domains {
                println!("{:?}", domain);

                if let Some(Ip::Public) = domain.ip {
                    let res = match ipify.ipv6().await {
                        Ok(ip) => Ok(ip),
                        Err(_) => ipify.ipv4().await,
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
