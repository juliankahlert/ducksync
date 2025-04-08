use reqwest::Client;
use std::collections::HashMap;

pub struct DuckDns {
    client: Client,
}

impl DuckDns {
    // Create a new DuckDns instance with a shared client
    pub fn new() -> Self {
        DuckDns {
            client: Client::new(),
        }
    }

    // Main entry point to update DuckDNS records (IP or TXT)
    pub async fn update(
        &self,
        domains: Vec<String>,
        token: String,
        ip: Option<String>,
        ipv6: Option<String>,
        verbose: Option<bool>,
        clear: Option<bool>,
    ) -> Result<(), String> {
        let params = self.build_params(domains, token, ip, ipv6, verbose, clear)?;
        self.send_request(params).await
    }

    // Main entry point to update DuckDNS TXT record
    pub async fn update_txt(
        &self,
        domains: Vec<String>,
        token: String,
        txt: String,
        verbose: Option<bool>,
        clear: Option<bool>,
    ) -> Result<(), String> {
        let params = self.build_txt_params(domains, token, txt, verbose, clear)?;
        self.send_request(params).await
    }

    // Build parameters for updating IP and IPv6 records
    fn build_params(
        &self,
        domains: Vec<String>,
        token: String,
        ip: Option<String>,
        ipv6: Option<String>,
        verbose: Option<bool>,
        clear: Option<bool>,
    ) -> Result<HashMap<&'static str, String>, String> {
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

        Ok(params)
    }

    // Build parameters specifically for updating TXT records
    fn build_txt_params(
        &self,
        domains: Vec<String>,
        token: String,
        txt: String,
        verbose: Option<bool>,
        clear: Option<bool>,
    ) -> Result<HashMap<&'static str, String>, String> {
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

        Ok(params)
    }

    // Send the HTTP request to the DuckDNS API and handle the response
    async fn send_request(&self, params: HashMap<&'static str, String>) -> Result<(), String> {
        let url = "https://www.duckdns.org/update";

        let response = self
            .client
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
