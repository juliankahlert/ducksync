use async_trait::async_trait;
use std::collections::HashMap;

#[async_trait]
pub trait DuckDnsClient: Send + Sync {
    async fn send_request(&self, params: HashMap<&'static str, String>) -> Result<String, String>;
}

#[async_trait]
impl DuckDnsClient for reqwest::Client {
    async fn send_request(&self, params: HashMap<&'static str, String>) -> Result<String, String> {
        let url = "https://www.duckdns.org/update";
        let response = self
            .get(url)
            .query(&params)
            .send()
            .await
            .map_err(|e| format!("Error sending request: {}", e))?;

        let body = response
            .text()
            .await
            .map_err(|e| format!("Error reading response body: {}", e))?;

        Ok(body)
    }
}

pub struct DuckDns<T: DuckDnsClient> {
    client: T,
}

pub type DefaultDuckDns = DuckDns<reqwest::Client>;

impl Default for DefaultDuckDns {
    fn default() -> Self {
        DuckDns::new(reqwest::Client::new())
    }
}

impl<T: DuckDnsClient> DuckDns<T> {
    pub fn new(client: T) -> Self {
        DuckDns { client }
    }

    pub async fn update(
        &self,
        domains: &[String],
        token: String,
        ip: Option<String>,
        ipv6: Option<String>,
        verbose: Option<bool>,
        clear: Option<bool>,
    ) -> Result<(), String> {
        let params = self.build_params(domains, token, ip, ipv6, verbose, clear)?;
        let response = self.client.send_request(params).await?;

        if response.starts_with("OK") {
            Ok(())
        } else {
            Err(format!("Update failed: {}", response))
        }
    }

    pub async fn update_txt(
        &self,
        domains: &[String],
        token: String,
        txt: String,
        verbose: Option<bool>,
        clear: Option<bool>,
    ) -> Result<(), String> {
        let params = self.build_txt_params(domains, token, txt, verbose, clear)?;
        let response = self.client.send_request(params).await?;

        if response.starts_with("OK") {
            Ok(())
        } else {
            Err(format!("Update failed: {}", response))
        }
    }

    // Build parameters for updating IP and IPv6 records
    fn build_params(
        &self,
        domains: &[String],
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
        domains: &[String],
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockDuckClient;

    #[async_trait::async_trait]
    impl DuckDnsClient for MockDuckClient {
        async fn send_request(
            &self,
            params: HashMap<&'static str, String>,
        ) -> Result<String, String> {
            if params.get("token") == Some(&"bad-token".to_string()) {
                Ok("KO".to_string())
            } else {
                Ok("OK".to_string())
            }
        }
    }

    #[tokio::test]
    async fn test_update_success() {
        let client = MockDuckClient;
        let duck = DuckDns::new(client);
        let result = duck
            .update(
                &vec!["example".to_string()],
                "test-token".to_string(),
                Some("1.2.3.4".to_string()),
                None,
                Some(true),
                None,
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_failure() {
        let client = MockDuckClient;
        let duck = DuckDns::new(client);
        let result = duck
            .update(
                &vec!["example".to_string()],
                "bad-token".to_string(),
                None,
                None,
                None,
                None,
            )
            .await;

        assert!(result.is_err());
    }
}
