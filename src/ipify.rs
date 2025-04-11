use async_trait::async_trait;
use std::net::IpAddr;

const IPIFY_API4: &str = "https://api.ipify.org?format=text";
const IPIFY_API6: &str = "https://api6.ipify.org?format=text";

#[async_trait]
pub trait IpFetcher: Send + Sync {
    async fn fetch_ip(&self, url: &str) -> Result<String, String>;
}

#[async_trait]
impl IpFetcher for reqwest::Client {
    async fn fetch_ip(&self, url: &str) -> Result<String, String> {
        self.get(url)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())
    }
}

pub struct Ipify<F: IpFetcher> {
    fetcher: F,
}

impl Default for Ipify<reqwest::Client> {
    fn default() -> Self {
        Ipify::new(reqwest::Client::new())
    }
}

impl<F: IpFetcher> Ipify<F> {
    pub fn new(fetcher: F) -> Self {
        Ipify { fetcher }
    }

    pub async fn ipv4(&self) -> Result<IpAddr, String> {
        self.get(IPIFY_API4).await
    }

    pub async fn ipv6(&self) -> Result<IpAddr, String> {
        self.get(IPIFY_API6).await
    }

    async fn get(&self, url: &str) -> Result<IpAddr, String> {
        let ip_str = self.fetcher.fetch_ip(url).await?;
        ip_str.parse::<IpAddr>().map_err(|e| e.to_string())
    }
}
