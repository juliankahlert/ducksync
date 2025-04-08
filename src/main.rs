use clap::{Parser, Subcommand};
use ducksync::config::{Config, Ip};
use ducksync::duckdns::DuckDns;
use ducksync::ipify::Ipify;

#[derive(Parser, Debug)]
#[command(name = "ducksync")]
struct Args {
    #[command(subcommand)]
    command: Option<Subs>,
}

#[derive(Subcommand, Debug, Clone)]
enum Subs {
    UpdateDns {
        #[arg(
            action = clap::ArgAction::Append,
            required = true,
            short,
            long,
        )]
        domain: Vec<String>,
        #[arg(short, long)]
        token: String,
    },
    UpdateTxt {
        #[arg(
            action = clap::ArgAction::Append,
            required = true,
            short,
            long,
        )]
        domain: Vec<String>,
        #[arg(short, long)]
        token: String,
    },
    Config {
        config: String,
    },
}

async fn update_dns_cmd(domains: Vec<String>, token: String) {
    let duckdns = DuckDns::new();
    let ipify = Ipify::new();

    let res = match ipify.ipv6().await {
        Ok(ip) => Ok(ip),
        Err(_) => ipify.ipv4().await,
    };

    let Ok(ip) = res else {
        return;
    };

    println!("IP found for domains {:?} => {}", domains, &ip);
    let res = duckdns
        .update(
            domains,
            token.clone(),
            Some(ip.to_string()),
            None,
            None,
            None,
        )
        .await;
    println!("{:?}", res);
}

async fn config_cmd(file: Option<String>) {
    let cfg = match Config::load(file).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            return;
        }
    };

    let duckdns = DuckDns::new();
    let ipify = Ipify::new();

    println!("Successfully loaded config:");
    for domain in cfg.domains {
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

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if let Some(sub) = args.command {
        match sub {
            Subs::Config { config } => config_cmd(Some(config)).await,
            Subs::UpdateDns { domain, token } => update_dns_cmd(domain, token).await,
            _ => return,
        }
        return;
    }

    config_cmd(None).await
}
