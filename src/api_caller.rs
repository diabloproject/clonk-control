use reqwest::{Client, cookie::Jar};
use std::sync::Arc;
use std::fs;
use serde_json::json;
use std::path::Path;
#[derive(Clone)]
pub struct ApiCaller {
    client: Client,
}

impl ApiCaller {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Load cookies from file
        let cookie_jar = Jar::default();
        if Path::new("cookie.txt").exists() {
            let cookie_content = fs::read_to_string("cookie.txt")?;
            // Add parsing logic for cookies here
            // This is a placeholder and may need to be adapted based on your specific cookie format
        }

        let client = Client::builder()
            .cookie_store(true)
            .cookie_provider(Arc::new(cookie_jar))
            .build()?;

        Ok(Self { client })
    }

    pub async fn call_api(
        &self, 
        api_name: &str, 
        input: Option<String>
    ) -> Result<String, reqwest::Error> {
        // Replace with your actual API endpoint
        let url = "https://secur.colonq.computer/api/redeem";
        
        let body = json!({
            "name": api_name,
            "input": input.unwrap_or_else(|| "undefined".to_string())
        });

        let response = self.client.post(url)
            .form(&body)
            .send()
            .await?
            .text()
            .await?;

        Ok(response)
    }
}
