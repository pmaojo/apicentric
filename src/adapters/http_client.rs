//! An implementation of the `ContractHttpClient` port that uses the `reqwest`
//! crate to send HTTP requests.
//!
//! This module provides a `ReqwestHttpClientAdapter` that can be used to send
//! HTTP requests to a real API for contract testing. It supports retries,
//! timeouts, and custom headers.

use crate::domain::contract_testing::*;
use crate::domain::ports::contract::*;
use async_trait::async_trait;
use reqwest::{Client, Method, RequestBuilder};
use serde_json::Value;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// An adapter that uses the `reqwest` crate to send HTTP requests.
pub struct ReqwestHttpClientAdapter {
    client: Client,
    default_timeout: Duration,
    max_retries: u32,
    retry_delay: Duration,
}

impl ReqwestHttpClientAdapter {
    /// Creates a new `ReqwestHttpClientAdapter` with default settings.
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("apicentric-contract-tester/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            default_timeout: Duration::from_secs(30),
            max_retries: 3,
            retry_delay: Duration::from_millis(1000),
        }
    }

    /// Sets the default timeout for HTTP requests.
    ///
    /// # Arguments
    ///
    /// * `timeout` - The timeout duration.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = timeout;
        self.client = Client::builder()
            .timeout(timeout)
            .user_agent("apicentric-contract-tester/1.0")
            .build()
            .expect("Failed to create HTTP client with custom timeout");
        self
    }

    /// Sets the retry policy for HTTP requests.
    ///
    /// # Arguments
    ///
    /// * `max_retries` - The maximum number of retries.
    /// * `delay` - The delay between retries.
    pub fn with_retries(mut self, max_retries: u32, delay: Duration) -> Self {
        self.max_retries = max_retries;
        self.retry_delay = delay;
        self
    }

    async fn execute_with_retries(
        &self,
        request_builder: RequestBuilder,
    ) -> Result<reqwest::Response, HttpClientError> {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            if attempt > 0 {
                debug!(
                    "Retrying request (attempt {}/{})",
                    attempt + 1,
                    self.max_retries + 1
                );
                tokio::time::sleep(self.retry_delay).await;
            }

            // Clone the request for retry attempts
            let request = match request_builder.try_clone() {
                Some(req) => req,
                None => {
                    return Err(HttpClientError::RequestFailed(
                        "Failed to clone request for retry".to_string(),
                    ))
                }
            };

            match request.send().await {
                Ok(response) => {
                    debug!("Request successful on attempt {}", attempt + 1);
                    return Ok(response);
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.max_retries {
                        warn!(
                            "Request failed on attempt {}: {}",
                            attempt + 1,
                            last_error.as_ref().unwrap()
                        );
                    }
                }
            }
        }

        Err(HttpClientError::RequestFailed(format!(
            "Request failed after {} attempts: {}",
            self.max_retries + 1,
            last_error.unwrap()
        )))
    }

    fn build_request(
        &self,
        base_url: &ApiUrl,
        config: &RealApiConfig,
        scenario: &ValidationScenario,
    ) -> Result<RequestBuilder, HttpClientError> {
        // Construct full URL
        let full_url = format!("{}{}", base_url.as_str(), scenario.path);

        // Convert HttpMethod to reqwest::Method
        let method = match scenario.method {
            HttpMethod::GET => Method::GET,
            HttpMethod::POST => Method::POST,
            HttpMethod::PUT => Method::PUT,
            HttpMethod::DELETE => Method::DELETE,
            HttpMethod::PATCH => Method::PATCH,
            HttpMethod::HEAD => Method::HEAD,
            HttpMethod::OPTIONS => Method::OPTIONS,
        };

        let mut request = self.client.request(method, &full_url);

        // Add headers from scenario
        for (key, value) in &scenario.headers {
            request = request.header(key, value);
        }

        // Add headers from config
        for (key, value) in &config.headers {
            request = request.header(key, value);
        }

        // Add authentication if configured
        if let Some(auth_header) = &config.auth_header {
            request = request.header("Authorization", auth_header);
        }

        // Add query parameters
        if !scenario.query_params.is_empty() {
            request = request.query(&scenario.query_params);
        }

        // Add request body if present
        if let Some(body) = &scenario.request_body {
            match body {
                RequestBody::Json(json_value) => {
                    request = request.json(json_value);
                }
                RequestBody::Text(text) => {
                    request = request.body(text.clone());
                }
                RequestBody::FormData(form_data) => {
                    request = request.form(form_data);
                }
            }
        }

        // Set timeout from config or use default
        let timeout = config
            .timeout
            .as_ref()
            .map(|t| Duration::from_millis(t.as_millis()))
            .unwrap_or(self.default_timeout);

        request = request.timeout(timeout);

        Ok(request)
    }

    async fn parse_response(
        &self,
        response: reqwest::Response,
        start_time: Instant,
    ) -> Result<ApiResponse, HttpClientError> {
        let status_code = response.status().as_u16();
        let duration_ms = start_time.elapsed().as_millis() as u64;

        // Extract headers
        let mut headers = HashMap::new();
        for (key, value) in response.headers() {
            if let Ok(value_str) = value.to_str() {
                headers.insert(key.to_string(), value_str.to_string());
            }
        }

        // Get content type for response parsing
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|ct| ct.to_str().ok())
            .unwrap_or("")
            .to_lowercase();

        // Read response body
        let body_bytes = response.bytes().await.map_err(|e| {
            HttpClientError::InvalidResponse(format!("Failed to read response body: {}", e))
        })?;

        // Parse response body based on content type
        let body = if content_type.contains("application/json") {
            match serde_json::from_slice::<Value>(&body_bytes) {
                Ok(json) => ResponseBody::Json(json),
                Err(_) => {
                    // If JSON parsing fails, treat as text
                    let text = String::from_utf8_lossy(&body_bytes).to_string();
                    ResponseBody::Text(text)
                }
            }
        } else {
            let text = String::from_utf8_lossy(&body_bytes).to_string();
            ResponseBody::Text(text)
        };

        Ok(ApiResponse {
            status_code,
            headers,
            body,
            duration_ms,
        })
    }
}

#[async_trait]
impl ContractHttpClient for ReqwestHttpClientAdapter {
    /// Executes an HTTP request and returns an `ApiResponse`.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL of the API.
    /// * `config` - The configuration for the real API.
    /// * `scenario` - The validation scenario to execute.
    ///
    /// # Returns
    ///
    /// An `ApiResponse` if the request was successful, or an `HttpClientError`
    /// if it was not.
    async fn execute_request(
        &self,
        base_url: &ApiUrl,
        config: &RealApiConfig,
        scenario: &ValidationScenario,
    ) -> Result<ApiResponse, HttpClientError> {
        let start_time = Instant::now();

        debug!(
            "Executing request: {} {} against {}",
            scenario.method,
            scenario.path,
            base_url.as_str()
        );

        // Build request
        let request = self.build_request(base_url, config, scenario)?;

        // Execute with retries
        let response = self.execute_with_retries(request).await?;

        // Parse response
        let api_response = self.parse_response(response, start_time).await?;

        info!(
            "Request completed: {} {} - Status: {} - Duration: {}ms",
            scenario.method, scenario.path, api_response.status_code, api_response.duration_ms
        );

        Ok(api_response)
    }

    /// Performs a health check on the API.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL of the API.
    ///
    /// # Returns
    ///
    /// `true` if the API is healthy, `false` otherwise.
    async fn health_check(&self, base_url: &ApiUrl) -> Result<bool, HttpClientError> {
        debug!("Performing health check for: {}", base_url.as_str());

        // Try a simple GET request to the base URL or a common health endpoint
        let health_urls = vec![
            format!("{}/health", base_url.as_str()),
            format!("{}/ping", base_url.as_str()),
            format!("{}/_health", base_url.as_str()),
            base_url.as_str().to_string(),
        ];

        for url in health_urls {
            match self
                .client
                .get(&url)
                .timeout(Duration::from_secs(5))
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() || response.status().as_u16() == 404 {
                        // 404 is also acceptable - means server is responding
                        info!("Health check successful for: {}", base_url.as_str());
                        return Ok(true);
                    }
                }
                Err(e) => {
                    debug!("Health check failed for {}: {}", url, e);
                    continue;
                }
            }
        }

        warn!("Health check failed for: {}", base_url.as_str());
        Ok(false)
    }
}

impl Default for ReqwestHttpClientAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// A builder for creating `ReqwestHttpClientAdapter` instances.
pub struct HttpClientBuilder {
    timeout: Duration,
    max_retries: u32,
    retry_delay: Duration,
    user_agent: String,
    default_headers: HashMap<String, String>,
}

impl HttpClientBuilder {
    /// Creates a new `HttpClientBuilder` with default settings.
    pub fn new() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_retries: 3,
            retry_delay: Duration::from_millis(1000),
            user_agent: "apicentric-contract-tester/1.0".to_string(),
            default_headers: HashMap::new(),
        }
    }

    /// Sets the timeout for HTTP requests.
    ///
    /// # Arguments
    ///
    /// * `timeout` - The timeout duration.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets the retry policy for HTTP requests.
    ///
    /// # Arguments
    ///
    /// * `max_retries` - The maximum number of retries.
    /// * `delay` - The delay between retries.
    pub fn retries(mut self, max_retries: u32, delay: Duration) -> Self {
        self.max_retries = max_retries;
        self.retry_delay = delay;
        self
    }

    /// Sets the user agent for HTTP requests.
    ///
    /// # Arguments
    ///
    /// * `user_agent` - The user agent string.
    pub fn user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = user_agent;
        self
    }

    /// Adds a default header to be sent with every HTTP request.
    ///
    /// # Arguments
    ///
    /// * `key` - The header name.
    /// * `value` - The header value.
    pub fn default_header(mut self, key: String, value: String) -> Self {
        self.default_headers.insert(key, value);
        self
    }

    /// Builds a `ReqwestHttpClientAdapter` with the specified settings.
    pub fn build(self) -> ReqwestHttpClientAdapter {
        let mut client_builder = Client::builder()
            .timeout(self.timeout)
            .user_agent(&self.user_agent);

        // Add default headers
        let mut default_headers = reqwest::header::HeaderMap::new();
        for (key, value) in self.default_headers {
            if let (Ok(header_name), Ok(header_value)) = (
                reqwest::header::HeaderName::from_bytes(key.as_bytes()),
                reqwest::header::HeaderValue::from_str(&value),
            ) {
                default_headers.insert(header_name, header_value);
            }
        }

        if !default_headers.is_empty() {
            client_builder = client_builder.default_headers(default_headers);
        }

        let client = client_builder
            .build()
            .expect("Failed to create HTTP client");

        ReqwestHttpClientAdapter {
            client,
            default_timeout: self.timeout,
            max_retries: self.max_retries,
            retry_delay: self.retry_delay,
        }
    }
}

impl Default for HttpClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_http_client_builder() {
        let client = HttpClientBuilder::new()
            .timeout(Duration::from_secs(10))
            .retries(2, Duration::from_millis(500))
            .user_agent("test-agent/1.0".to_string())
            .default_header("X-Custom-Header".to_string(), "test-value".to_string())
            .build();

        assert_eq!(client.default_timeout, Duration::from_secs(10));
        assert_eq!(client.max_retries, 2);
        assert_eq!(client.retry_delay, Duration::from_millis(500));
    }

    #[test]
    fn test_build_request() {
        let client = ReqwestHttpClientAdapter::new();
        let base_url = ApiUrl::new("https://api.example.com".to_string()).unwrap();
        let config = RealApiConfig {
            environment: "test".to_string(),
            base_url: base_url.clone(),
            auth_header: Some("Bearer token123".to_string()),
            headers: {
                let mut headers = HashMap::new();
                headers.insert("X-API-Key".to_string(), "test-key".to_string());
                headers
            },
            timeout: Some(TimeoutDuration::new(5000).unwrap()),
            retry_attempts: RetryAttempts::new(2).unwrap(),
        };

        let scenario = ValidationScenario::new(
            "test-scenario".to_string(),
            "/api/users".to_string(),
            HttpMethod::GET,
        )
        .with_headers({
            let mut headers = HashMap::new();
            headers.insert("Accept".to_string(), "application/json".to_string());
            headers
        });

        let request = client.build_request(&base_url, &config, &scenario);
        assert!(request.is_ok());
    }
}
