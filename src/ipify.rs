use reqwest::Client;
use std::net::IpAddr;

const IPIFY_API4: &'static str = "https://api.ipify.org?format=text";
const IPIFY_API6: &'static str = "https://api6.ipify.org?format=text";

pub struct Ipify {
    client: Client,
}

impl Ipify {
    pub fn new() -> Self {
        Ipify {
            client: Client::new(),
        }
    }

    pub async fn ipv4(&self) -> Result<IpAddr, String> {
        self.get(IPIFY_API4).await
    }

    pub async fn ipv6(&self) -> Result<IpAddr, String> {
        self.get(IPIFY_API6).await
    }

    async fn get(&self, url: &str) -> Result<IpAddr, String> {
        let public_ip: String = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())?;

        public_ip.parse::<IpAddr>().map_err(|e| e.to_string())
    }
}
