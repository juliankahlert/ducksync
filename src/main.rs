use clap::{ArgAction, Parser, Subcommand};
use ducksync::config::{Config, Ip};
use ducksync::duckdns::DuckDns;
use ducksync::ipify::Ipify;
use env_logger::Env;
use log::{error, info, trace};

#[derive(Parser, Debug)]
#[command(name = "ducksync")]
struct Args {
    #[command(subcommand)]
    command: Option<Subs>,
    /// Verbosity (-v, -vv, -vvv, etc.)
    #[arg(short, action = ArgAction::Count)]
    verbose: u8,
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

type IpMap = Vec<(Vec<String>, String)>;

async fn update_dns_cmd(domains: Vec<String>, token: String) -> Result<IpMap, String> {
    let duckdns = DuckDns::new();
    let ipify = Ipify::new();

    let res = match ipify.ipv6().await {
        Ok(ip) => Ok(ip),
        Err(_) => ipify.ipv4().await,
    };

    let Ok(ip) = res else {
        return Err("Failed to get public ip".to_string());
    };

    duckdns
        .update(
            &domains,
            token.clone(),
            Some(ip.to_string()),
            None,
            None,
            None,
        )
        .await
        .map(|_| vec![(domains, ip.to_string())])
}

async fn config_cmd(file: Option<String>) -> Result<IpMap, String> {
    let cfg = match Config::load(file).await {
        Ok(c) => c,
        Err(e) => {
            return Err(format!("Error loading config: {}", e));
        }
    };

    let mut map = vec![];
    let duckdns = DuckDns::new();
    let ipify = Ipify::new();

    for domain in cfg.domains {
        if let Some(Ip::Public) = domain.ip {
            let res = match ipify.ipv6().await {
                Ok(ip) => Ok(ip),
                Err(e) => {
                    trace!("Could not resolve IPv6: {}", e);
                    ipify.ipv4().await
                },
            };

            let Ok(ip) = res else {
                return Err("Failed to get public ip".to_string());
            };

            let domains = vec![domain.name.clone()];
            if let Err(e) = duckdns
                .update(
                    &domains,
                    domain.token,
                    Some(ip.to_string()),
                    None,
                    None,
                    None,
                )
                .await
            {
                error!("{}", e);
                continue;
            }

            map.push((domains, ip.to_string()));
        }
    }

    Ok(map)
}

fn print_res(res: Result<IpMap, String>) -> bool {
    match res {
        Err(e) => {
            error!("{}", e);
            false
        }
        Ok(ip_map) => {
            for m in ip_map {
                info!("{:?} => {}", m.0, m.1);
            }
            true
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let log_level = match args.verbose {
        0 => "",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    env_logger::Builder::from_env(Env::default().default_filter_or(log_level)).init();

    let ok = if let Some(sub) = args.command {
        let res = match sub {
            Subs::Config { config } => config_cmd(Some(config)).await,
            Subs::UpdateDns { domain, token } => update_dns_cmd(domain, token).await,
            _ => Err("Not supported".to_string()),
        };

        print_res(res)
    } else {
        let res = config_cmd(None).await;
        print_res(res)
    };

    if !ok {
        std::process::exit(1);
    }
}
