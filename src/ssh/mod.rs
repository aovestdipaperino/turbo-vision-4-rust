// (C) 2025 - Enzo Lombardi

//! SSH server support for turbo-vision applications.
//!
//! This module provides infrastructure for serving turbo-vision TUI applications
//! over SSH connections using the russh library.
//!
//! # Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────────┐
//! │                        SSH Server                                │
//! ├──────────────────────────────────────────────────────────────────┤
//! │                                                                  │
//! │  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐          │
//! │  │ SSH Client  │    │ SSH Client  │    │ SSH Client  │   ...    │
//! │  │ Connection  │    │ Connection  │    │ Connection  │          │
//! │  └──────┬──────┘    └──────┬──────┘    └──────┬──────┘          │
//! │         │                  │                  │                  │
//! │         ▼                  ▼                  ▼                  │
//! │  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐          │
//! │  │ TuiHandler  │    │ TuiHandler  │    │ TuiHandler  │          │
//! │  │ (per conn)  │    │ (per conn)  │    │ (per conn)  │          │
//! │  └──────┬──────┘    └──────┬──────┘    └──────┬──────┘          │
//! │         │                  │                  │                  │
//! │         ▼                  ▼                  ▼                  │
//! │  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐          │
//! │  │   TUI App   │    │   TUI App   │    │   TUI App   │          │
//! │  │ (Terminal)  │    │ (Terminal)  │    │ (Terminal)  │          │
//! │  └─────────────┘    └─────────────┘    └─────────────┘          │
//! │                                                                  │
//! └──────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use turbo_vision::ssh::{SshServer, SshServerConfig};
//! use turbo_vision::Terminal;
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = SshServerConfig::new()
//!         .bind_addr("0.0.0.0:2222");
//!
//!     let server = SshServer::new(config, |backend| {
//!         // Create your TUI application with the SSH backend
//!         let terminal = Terminal::with_backend(backend).unwrap();
//!         // Run your app...
//!     });
//!
//!     server.run().await.unwrap();
//! }
//! ```

mod handler;
mod server;

pub use handler::{TuiHandler, TuiSession};
pub use server::{SshServer, SshServerConfig, AppFactory};
