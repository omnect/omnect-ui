use anyhow::{Context, Result, ensure};
use reqwest::{Client, Response};
use std::{path::Path, sync::OnceLock};

/// Factory for creating configured HTTP clients
///
/// This module centralizes HTTP client creation to ensure consistent
/// configuration across the application. It provides three types of clients:
/// - Unix socket clients for local service communication
/// - HTTPS clients for external API calls
/// - Workload socket clients for IoT Edge workload API
pub struct HttpClientFactory;

static HTTPS_CLIENT: OnceLock<Client> = OnceLock::new();

impl HttpClientFactory {
    /// Get or create an HTTPS client
    ///
    /// Returns a shared, cached HTTPS client instance. This client is reused
    /// across the application to share connection pools.
    pub fn https_client() -> &'static Client {
        HTTPS_CLIENT.get_or_init(Client::new)
    }

    /// Create a Unix socket client for local service communication
    ///
    /// # Arguments
    /// * `socket_path` - Path to the Unix socket
    ///
    /// # Examples
    /// ```no_run
    /// use omnect_ui::http_client::HttpClientFactory;
    /// use std::path::Path;
    ///
    /// let client = HttpClientFactory::unix_socket_client(Path::new("/socket/api.sock"))
    ///     .expect("failed to create client");
    /// ```
    pub fn unix_socket_client(socket_path: &Path) -> Result<Client> {
        Client::builder()
            .unix_socket(socket_path)
            .build()
            .context("failed to create Unix socket HTTP client")
    }

    /// Create a Unix socket client for IoT Edge workload API
    ///
    /// This is specifically for communicating with the IoT Edge workload API
    /// over a Unix socket.
    ///
    /// # Arguments
    /// * `workload_uri` - The workload URI (e.g., "unix:///var/run/iotedge/workload.sock")
    ///
    /// # Examples
    /// ```no_run
    /// use omnect_ui::http_client::HttpClientFactory;
    ///
    /// let client = HttpClientFactory::workload_client("unix:///var/run/iotedge/workload.sock")
    ///     .expect("failed to create workload client");
    /// ```
    #[cfg_attr(feature = "mock", allow(dead_code))]
    pub fn workload_client(workload_uri: &str) -> Result<Client> {
        let socket_path = workload_uri
            .strip_prefix("unix://")
            .context("workload URI must use unix:// scheme")?;

        Client::builder()
            .unix_socket(socket_path)
            .build()
            .context("failed to create workload socket HTTP client")
    }
}

/// Handle HTTP response by checking status and extracting body
///
/// This is a common utility for processing HTTP responses across all HTTP clients.
/// It ensures the response status is successful and extracts the body text.
///
/// # Arguments
/// * `res` - The HTTP response to handle
/// * `context_msg` - Context message describing the request (e.g., "certificate request")
///
/// # Returns
/// * `Ok(String)` - The response body if the status is successful
/// * `Err` - If the status is not successful or reading the body fails
pub async fn handle_http_response(res: Response, context_msg: &str) -> Result<String> {
    let status = res.status();
    let body = res.text().await.context("failed to read response body")?;

    ensure!(
        status.is_success(),
        "{} failed with status {} and body: {}",
        context_msg,
        status,
        body
    );

    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_https_client_is_cached() {
        let client1 = HttpClientFactory::https_client();
        let client2 = HttpClientFactory::https_client();

        // Should return the same instance (same pointer)
        assert!(std::ptr::eq(client1, client2));
    }

    #[test]
    fn test_https_client_returns_valid_client() {
        let client = HttpClientFactory::https_client();
        // Verify that we get a valid Client reference
        // If this compiles and runs, we have a valid client
        let client_ptr = client as *const Client;
        assert!(!client_ptr.is_null());
    }

    #[test]
    fn test_https_client_multiple_calls_same_instance() {
        // Test that multiple sequential calls return the same cached instance
        let client1 = HttpClientFactory::https_client();
        let client2 = HttpClientFactory::https_client();
        let client3 = HttpClientFactory::https_client();

        assert!(std::ptr::eq(client1, client2));
        assert!(std::ptr::eq(client2, client3));
        assert!(std::ptr::eq(client1, client3));
    }

    #[test]
    fn test_workload_client_parses_uri() {
        let result = HttpClientFactory::workload_client("unix:///var/run/iotedge/workload.sock");
        assert!(result.is_ok());
    }

    #[test]
    fn test_workload_client_rejects_invalid_scheme() {
        let result = HttpClientFactory::workload_client("http://localhost:8080");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("unix://"));
    }

    #[test]
    fn test_unix_socket_client_creates_client() {
        let socket_path = Path::new("/tmp/test.sock");
        let result = HttpClientFactory::unix_socket_client(socket_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unix_socket_client_with_relative_path() {
        let socket_path = Path::new("relative/path/test.sock");
        let result = HttpClientFactory::unix_socket_client(socket_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unix_socket_client_with_empty_path() {
        let socket_path = Path::new("");
        let result = HttpClientFactory::unix_socket_client(socket_path);
        // This should succeed in creating the client, even though the path is empty
        // The actual connection will fail later when attempting to use it
        assert!(result.is_ok());
    }
}
