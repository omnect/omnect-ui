use anyhow::{Context, Result};
use reqwest::Client;
use std::path::Path;

/// Factory for creating configured HTTP clients
///
/// This module centralizes HTTP client creation to ensure consistent
/// configuration across the application. It provides two types of clients:
/// - Unix socket clients for local service communication
/// - Workload socket clients for IoT Edge workload API
pub struct HttpClientFactory;

impl HttpClientFactory {
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

        Self::unix_socket_client(Path::new(socket_path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workload_client_parses_uri() {
        let result = HttpClientFactory::workload_client("unix:///var/run/iotedge/workload.sock");
        // This should succeed in creating the client, even if the socket doesn't exist
        // The actual connection will fail later when attempting to use it
        assert!(result.is_ok());
    }

    #[test]
    fn test_workload_client_rejects_invalid_scheme() {
        let result = HttpClientFactory::workload_client("http://localhost:8080");
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert_eq!(error_message, "workload URI must use unix:// scheme");
    }

    #[test]
    fn test_unix_socket_client_creates_client() {
        let socket_path = Path::new("/tmp/test.sock");
        let result = HttpClientFactory::unix_socket_client(socket_path);
        // This should succeed in creating the client, even if the socket doesn't exist
        // The actual connection will fail later when attempting to use it
        assert!(result.is_ok());
    }

    #[test]
    fn test_unix_socket_client_with_relative_path() {
        let socket_path = Path::new("relative/path/test.sock");
        let result = HttpClientFactory::unix_socket_client(socket_path);
        // This should succeed in creating the client, even if the socket doesn't exist
        // The actual connection will fail later when attempting to use it
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
