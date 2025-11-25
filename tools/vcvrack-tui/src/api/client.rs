//! HTTP client for VCV Rack API

use anyhow::{Context, Result};
use reqwest::Client;
use std::time::Duration;

use super::types::*;

/// HTTP client wrapper for VCV Rack API
#[derive(Clone)]
pub struct RackApiClient {
    client: Client,
    base_url: String,
}

impl RackApiClient {
    /// Create a new API client
    pub fn new(host: &str, port: u16, timeout_ms: u64) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_millis(timeout_ms))
            .build()
            .context("Failed to create HTTP client")?;

        let base_url = format!("http://{}:{}", host, port);

        Ok(Self { client, base_url })
    }

    /// Get base URL for display
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// List all available plugins
    pub async fn get_plugins(&self) -> Result<Vec<Plugin>> {
        let url = format!("{}/api/plugins", self.base_url);
        let response: PluginsResponse = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to connect to VCV Rack")?
            .json()
            .await
            .context("Failed to parse plugins response")?;
        Ok(response.plugins)
    }

    /// List all available module models
    pub async fn get_models(&self) -> Result<Vec<AvailableModel>> {
        let url = format!("{}/api/models", self.base_url);
        let response: ModelsResponse = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to connect to VCV Rack")?
            .json()
            .await
            .context("Failed to parse models response")?;
        Ok(response.models)
    }

    /// List all modules in the current patch
    pub async fn get_modules(&self) -> Result<Vec<PatchModule>> {
        let url = format!("{}/api/modules", self.base_url);
        let response: ModulesResponse = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to connect to VCV Rack")?
            .json()
            .await
            .context("Failed to parse modules response")?;
        Ok(response.modules)
    }

    /// Get detailed information about a specific module
    pub async fn get_module_details(&self, id: i64) -> Result<ModuleDetails> {
        let url = format!("{}/api/modules/{}", self.base_url, id);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to connect to VCV Rack")?;

        if response.status().is_success() {
            response
                .json()
                .await
                .context("Failed to parse module details response")
        } else {
            let error: ErrorResponse = response
                .json()
                .await
                .unwrap_or(ErrorResponse {
                    error: "Unknown error".to_string(),
                });
            anyhow::bail!("API error: {}", error.error)
        }
    }

    /// Add a new module to the patch
    pub async fn create_module(&self, request: CreateModuleRequest) -> Result<CreateModuleResponse> {
        let url = format!("{}/api/modules", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to connect to VCV Rack")?;

        if response.status().is_success() {
            response
                .json()
                .await
                .context("Failed to parse create module response")
        } else {
            let error: ErrorResponse = response
                .json()
                .await
                .unwrap_or(ErrorResponse {
                    error: "Unknown error".to_string(),
                });
            anyhow::bail!("API error: {}", error.error)
        }
    }

    /// Remove a module from the patch
    pub async fn delete_module(&self, id: i64) -> Result<()> {
        let url = format!("{}/api/modules/{}", self.base_url, id);
        let response = self
            .client
            .delete(&url)
            .send()
            .await
            .context("Failed to connect to VCV Rack")?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error: ErrorResponse = response
                .json()
                .await
                .unwrap_or(ErrorResponse {
                    error: "Unknown error".to_string(),
                });
            anyhow::bail!("API error: {}", error.error)
        }
    }

    /// List all cables in the current patch
    pub async fn get_cables(&self) -> Result<Vec<Cable>> {
        let url = format!("{}/api/cables", self.base_url);
        let response: CablesResponse = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to connect to VCV Rack")?
            .json()
            .await
            .context("Failed to parse cables response")?;
        Ok(response.cables)
    }

    /// Create a new cable connection
    pub async fn create_cable(&self, request: CreateCableRequest) -> Result<CreateCableResponse> {
        let url = format!("{}/api/cables", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to connect to VCV Rack")?;

        if response.status().is_success() {
            response
                .json()
                .await
                .context("Failed to parse create cable response")
        } else {
            let error: ErrorResponse = response
                .json()
                .await
                .unwrap_or(ErrorResponse {
                    error: "Unknown error".to_string(),
                });
            anyhow::bail!("API error: {}", error.error)
        }
    }

    /// Remove a cable from the patch
    pub async fn delete_cable(&self, id: i64) -> Result<()> {
        let url = format!("{}/api/cables/{}", self.base_url, id);
        let response = self
            .client
            .delete(&url)
            .send()
            .await
            .context("Failed to connect to VCV Rack")?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error: ErrorResponse = response
                .json()
                .await
                .unwrap_or(ErrorResponse {
                    error: "Unknown error".to_string(),
                });
            anyhow::bail!("API error: {}", error.error)
        }
    }

    /// Check if the server is reachable
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/api/modules", self.base_url);
        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}
