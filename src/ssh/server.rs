// (C) 2025 - Enzo Lombardi

//! SSH server for turbo-vision TUI applications.
//!
//! This module provides an easy-to-use SSH server that can serve
//! turbo-vision applications to remote clients.

use std::net::SocketAddr;
use std::sync::Arc;

use russh::server::{Config, Server};
use russh_keys::PrivateKey;

use super::handler::TuiHandler;

/// Factory function type for creating TUI applications.
///
/// This function is called for each new SSH connection and receives
/// the backend to use for creating a Terminal.
pub type AppFactory = Box<dyn Fn(Box<dyn crate::terminal::Backend>) + Send + Sync>;

/// Configuration for the SSH server.
pub struct SshServerConfig {
    /// Address to bind the server to.
    pub bind_addr: String,
    /// SSH host keys.
    pub keys: Vec<PrivateKey>,
    /// Maximum number of concurrent connections.
    pub max_connections: Option<usize>,
}

impl SshServerConfig {
    /// Create a new server configuration with default values.
    pub fn new() -> Self {
        Self {
            bind_addr: "0.0.0.0:2222".to_string(),
            keys: Vec::new(),
            max_connections: None,
        }
    }

    /// Set the bind address.
    pub fn bind_addr(mut self, addr: impl Into<String>) -> Self {
        self.bind_addr = addr.into();
        self
    }

    /// Add a host key.
    pub fn add_key(mut self, key: PrivateKey) -> Self {
        self.keys.push(key);
        self
    }

    /// Generate a random Ed25519 host key.
    pub fn generate_key(mut self) -> Self {
        use rand::rngs::OsRng;
        if let Ok(key) = PrivateKey::random(&mut OsRng, ssh_key::Algorithm::Ed25519) {
            self.keys.push(key);
        }
        self
    }

    /// Set maximum concurrent connections.
    pub fn max_connections(mut self, max: usize) -> Self {
        self.max_connections = Some(max);
        self
    }

    /// Build the russh Config.
    fn build_russh_config(&self) -> Config {
        use rand::rngs::OsRng;
        let mut config = Config::default();

        if !self.keys.is_empty() {
            config.keys = self.keys.clone();
        } else {
            // Generate a key if none provided
            if let Ok(key) = PrivateKey::random(&mut OsRng, ssh_key::Algorithm::Ed25519) {
                config.keys = vec![key];
            }
        }

        config
    }
}

impl Default for SshServerConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// SSH server that serves turbo-vision TUI applications.
///
/// Each SSH connection gets its own TUI application instance.
///
/// # Example
///
/// ```rust,ignore
/// use turbo_vision::ssh::{SshServer, SshServerConfig};
/// use turbo_vision::Terminal;
///
/// #[tokio::main]
/// async fn main() {
///     let config = SshServerConfig::new()
///         .bind_addr("0.0.0.0:2222")
///         .generate_key();
///
///     let server = SshServer::with_factory(config, |backend| {
///         let mut terminal = Terminal::with_backend(backend).unwrap();
///         // Run your TUI application...
///     });
///
///     println!("SSH server listening on port 2222");
///     println!("Connect with: ssh -p 2222 user@localhost");
///
///     server.run().await.unwrap();
/// }
/// ```
pub struct SshServer<F>
where
    F: Fn() -> Box<dyn FnOnce(Box<dyn crate::terminal::Backend>) + Send> + Send + Sync + 'static,
{
    config: SshServerConfig,
    app_factory: Arc<F>,
}

impl<F> SshServer<F>
where
    F: Fn() -> Box<dyn FnOnce(Box<dyn crate::terminal::Backend>) + Send> + Send + Sync + 'static,
{
    /// Create a new SSH server with an application factory.
    ///
    /// The factory function is called for each new connection and should
    /// return a closure that will be run with the SSH backend.
    pub fn new(config: SshServerConfig, factory: F) -> Self {
        Self {
            config,
            app_factory: Arc::new(factory),
        }
    }

    /// Run the SSH server.
    ///
    /// This will block until the server is shut down.
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let russh_config = Arc::new(self.config.build_russh_config());
        let addr = &self.config.bind_addr;

        log::info!("Starting SSH server on {}", addr);

        let mut server = TuiServer {
            app_factory: self.app_factory,
        };

        server.run_on_address(russh_config, addr).await?;

        Ok(())
    }
}

/// Internal server implementation.
struct TuiServer<F>
where
    F: Fn() -> Box<dyn FnOnce(Box<dyn crate::terminal::Backend>) + Send> + Send + Sync + 'static,
{
    app_factory: Arc<F>,
}

impl<F> Server for TuiServer<F>
where
    F: Fn() -> Box<dyn FnOnce(Box<dyn crate::terminal::Backend>) + Send> + Send + Sync + 'static,
{
    type Handler = TuiHandler<Box<dyn FnOnce(Box<dyn crate::terminal::Backend>) + Send>>;

    fn new_client(&mut self, peer_addr: Option<SocketAddr>) -> Self::Handler {
        log::info!("New SSH connection from {:?}", peer_addr);
        let factory = (self.app_factory)();
        TuiHandler::new(factory, peer_addr)
    }
}

/// Convenience function to run a simple SSH TUI server.
///
/// # Example
///
/// ```rust,ignore
/// use turbo_vision::ssh::run_ssh_server;
/// use turbo_vision::Terminal;
///
/// #[tokio::main]
/// async fn main() {
///     run_ssh_server("0.0.0.0:2222", || {
///         Box::new(|backend| {
///             let mut terminal = Terminal::with_backend(backend).unwrap();
///             // Run your TUI application...
///         })
///     }).await.unwrap();
/// }
/// ```
pub async fn run_ssh_server<F>(
    addr: &str,
    app_factory: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn() -> Box<dyn FnOnce(Box<dyn crate::terminal::Backend>) + Send> + Send + Sync + 'static,
{
    let config = SshServerConfig::new()
        .bind_addr(addr)
        .generate_key();

    let server = SshServer::new(config, app_factory);
    server.run().await
}
