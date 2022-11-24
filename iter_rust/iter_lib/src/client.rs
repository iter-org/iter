use serde::{Serialize, de::DeserializeOwned};

#[derive(Debug, Clone)]
pub struct Client {
    endpoint: String,
}

impl Client {
    pub fn new(endpoint: String) -> Self {
        Self { endpoint }
    }

    async fn req_api<T: Serialize + DeserializeOwned>(&self, req: &str) -> Result<T, anyhow::Error> {
        let url = format!("https://{}/graph", self.endpoint);
        let client = reqwest::Client::new();
        let request = client
            .request(reqwest::Method::POST, &url)
            .body(serde_json::json!({
                "query": req
            }).to_string())
            .header("Content-Type", "application/json")
            .header("Accept", "application/json");

        let res = request.send().await?;
        
        let data = res.json::<T>().await?;

        Ok(data)
    }

    pub async fn ping(&self) -> Result<String, anyhow::Error> {
        let res: serde_json::Value = self.req_api("message { ping }").await?;

        match res["data"]["message"]["ping"].as_str() {
            Some(ping) => Ok(ping.to_string()),
            None => Err(anyhow::anyhow!("no ping received"))
        }
    }
}