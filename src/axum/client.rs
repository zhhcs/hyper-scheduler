use crate::{axum::get_port, runwasm::RegisterConfig};

use super::{CallConfigRequest, TestRequest};

pub struct Client {
    client: reqwest::Client,
}

impl Client {
    pub fn new() -> Client {
        Client {
            client: reqwest::Client::new(),
        }
    }

    pub async fn init(&self, config: &RegisterConfig) -> Result<(), reqwest::Error> {
        let url = format!("http://127.0.0.1:{}/init", get_port());
        let json = serde_json::to_string(config).unwrap();
        let resp = self
            .client
            .get(url)
            .header("Content-Type", "application/json")
            .body(json.clone())
            .send()
            .await?;

        let body = resp.text().await?;
        tracing::info!("Response: {}", body);
        Ok(())
    }

    pub async fn test(&self, test_request: TestRequest) -> Result<(), reqwest::Error> {
        let url = format!("http://127.0.0.1:{}/test", get_port());
        let json = serde_json::to_string(&test_request).unwrap();
        let resp = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .body(json.clone())
            .send()
            .await?;

        let body = resp.text().await?;
        tracing::info!("Response: {}", body);
        Ok(())
    }

    pub async fn call(&self, call_config: &CallConfigRequest) -> Result<(), reqwest::Error> {
        let url = format!("http://127.0.0.1:{}/call", get_port());
        let json = serde_json::to_string(call_config).unwrap();

        let resp = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .body(json.clone())
            .send()
            .await?;

        let body = resp.text().await?;
        tracing::info!("Response: {}", body);
        Ok(())
    }

    pub async fn get_status_by_name(&self, uname: &str) -> Result<(), reqwest::Error> {
        let url = format!("http://127.0.0.1:{}/uname", get_port());

        let resp = self
            .client
            .get(url)
            .query(&[("uname", uname)])
            .send()
            .await?;

        let body = resp.text().await?;
        tracing::info!("Response: {}", body);
        Ok(())
    }

    pub async fn get_status(&self) -> Result<(), reqwest::Error> {
        let url = format!("http://127.0.0.1:{}/status", get_port());

        let resp = self.client.get(url).send().await?;

        let body = resp.text().await?;
        tracing::info!("Response: {}", body);
        Ok(())
    }
}
