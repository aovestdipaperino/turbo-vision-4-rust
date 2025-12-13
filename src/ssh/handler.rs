// (C) 2025 - Enzo Lombardi

//! SSH connection handler for TUI sessions.
//!
//! This module provides the russh handler implementation that bridges
//! SSH I/O to turbo-vision applications.

use std::sync::Arc;

use async_trait::async_trait;
use parking_lot::Mutex;
use russh::server::{Auth, Handler, Handle, Msg, Session};
use russh::{Channel, ChannelId, CryptoVec};
use russh_keys::PublicKey;
use tokio::sync::mpsc;

use crate::terminal::{InputParser, SshSessionHandle};

/// Forward output from TUI to SSH channel.
///
/// This runs as a background task, continuously reading output from the TUI
/// and sending it to the SSH client. When the TUI exits (output_rx closes),
/// this task sends EOF and closes the channel.
async fn forward_output(
    handle: Handle,
    channel_id: ChannelId,
    mut output_rx: mpsc::UnboundedReceiver<Vec<u8>>,
) {
    // Forward output until the TUI exits (channel closes)
    while let Some(data) = output_rx.recv().await {
        if !data.is_empty() {
            if handle.data(channel_id, CryptoVec::from(data)).await.is_err() {
                break;
            }
        }
    }

    // TUI has exited - close the SSH channel
    let _ = handle.eof(channel_id).await;
    let _ = handle.close(channel_id).await;
}

/// A TUI session for a single SSH connection.
pub struct TuiSession {
    /// The SSH channel ID.
    pub channel_id: ChannelId,
    /// Handle for communicating with the TUI.
    pub handle: SshSessionHandle,
    /// Output receiver (moved to forwarding task when shell starts).
    pub output_rx: Option<mpsc::UnboundedReceiver<Vec<u8>>>,
    /// Backend (moved to TUI when shell starts).
    pub backend: Option<crate::terminal::SshBackend>,
}

/// SSH handler that manages TUI sessions.
///
/// Implements the russh `Handler` trait to handle SSH protocol events
/// and route them to the TUI application.
pub struct TuiHandler<F>
where
    F: FnOnce(Box<dyn crate::terminal::Backend>) + Send + 'static,
{
    session: Option<TuiSession>,
    app_factory: Option<F>,
    peer_addr: Option<std::net::SocketAddr>,
}

impl<F> TuiHandler<F>
where
    F: FnOnce(Box<dyn crate::terminal::Backend>) + Send + 'static,
{
    /// Create a new TUI handler.
    pub fn new(app_factory: F, peer_addr: Option<std::net::SocketAddr>) -> Self {
        Self {
            session: None,
            app_factory: Some(app_factory),
            peer_addr,
        }
    }

    /// Start the TUI application (called by shell_request or exec_request).
    async fn start_tui(&mut self, channel_id: ChannelId, session: &mut Session) -> Result<(), russh::Error> {
        if let Some(ref mut s) = self.session {
            if s.channel_id == channel_id {
                // Start output forwarding first
                if let Some(output_rx) = s.output_rx.take() {
                    let handle = session.handle();
                    tokio::spawn(async move {
                        forward_output(handle, channel_id, output_rx).await;
                    });
                }

                // Now spawn the TUI application
                if let Some(backend) = s.backend.take() {
                    if let Some(factory) = self.app_factory.take() {
                        tokio::task::spawn_blocking(move || {
                            factory(Box::new(backend));
                        });
                    }
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl<F> Handler for TuiHandler<F>
where
    F: FnOnce(Box<dyn crate::terminal::Backend>) + Send + 'static,
{
    type Error = russh::Error;

    /// Handle password authentication.
    ///
    /// Override this in your implementation to add real authentication.
    /// Default accepts all passwords (NOT SECURE - for demo only).
    async fn auth_password(
        &mut self,
        user: &str,
        _password: &str,
    ) -> Result<Auth, Self::Error> {
        log::info!("Password auth attempt from {:?} for user '{}'", self.peer_addr, user);
        // WARNING: This accepts all passwords - implement real auth!
        Ok(Auth::Accept)
    }

    /// Handle public key authentication.
    ///
    /// Override this in your implementation to add real authentication.
    /// Default accepts all keys (NOT SECURE - for demo only).
    async fn auth_publickey(
        &mut self,
        user: &str,
        _key: &PublicKey,
    ) -> Result<Auth, Self::Error> {
        log::info!("Pubkey auth attempt from {:?} for user '{}'", self.peer_addr, user);
        // WARNING: This accepts all keys - implement real auth!
        Ok(Auth::Accept)
    }

    /// Handle channel open request.
    async fn channel_open_session(
        &mut self,
        channel: Channel<Msg>,
        _session: &mut Session,
    ) -> Result<bool, Self::Error> {
        log::debug!("Channel open session request");

        // Create session components
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let (output_tx, output_rx) = mpsc::unbounded_channel();
        let size = Arc::new(Mutex::new((80u16, 24u16)));

        let backend = crate::terminal::SshBackend::new(
            output_tx,
            event_rx,
            Arc::clone(&size),
        );

        // Create a dummy output_rx for the handle (the real one goes to the forwarding task)
        let (_dummy_tx, dummy_rx) = mpsc::unbounded_channel();
        let handle = SshSessionHandle {
            event_tx,
            output_rx: dummy_rx,
            size,
            input_parser: InputParser::new(),
        };

        self.session = Some(TuiSession {
            channel_id: channel.id(),
            handle,
            output_rx: Some(output_rx),
            backend: Some(backend),
        });

        // Note: TUI will be spawned in shell_request after forwarding is set up

        Ok(true)
    }

    /// Handle PTY request.
    async fn pty_request(
        &mut self,
        channel_id: ChannelId,
        term: &str,
        col_width: u32,
        row_height: u32,
        _pix_width: u32,
        _pix_height: u32,
        _modes: &[(russh::Pty, u32)],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        log::debug!(
            "PTY request: {}x{} term={}",
            col_width, row_height, term
        );

        if let Some(ref mut s) = self.session {
            if s.channel_id == channel_id {
                s.handle.resize(col_width as u16, row_height as u16);
            }
        }

        session.channel_success(channel_id)?;
        Ok(())
    }

    /// Handle shell request.
    async fn shell_request(
        &mut self,
        channel_id: ChannelId,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        self.start_tui(channel_id, session).await?;
        session.channel_success(channel_id)?;
        Ok(())
    }

    /// Handle exec request (some clients send this instead of shell).
    async fn exec_request(
        &mut self,
        channel_id: ChannelId,
        _data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        // Treat exec request like shell request - start the TUI
        self.start_tui(channel_id, session).await?;
        session.channel_success(channel_id)?;
        Ok(())
    }

    /// Handle window size change.
    async fn window_change_request(
        &mut self,
        channel_id: ChannelId,
        col_width: u32,
        row_height: u32,
        _pix_width: u32,
        _pix_height: u32,
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        log::debug!("Window change: {}x{}", col_width, row_height);

        if let Some(ref mut s) = self.session {
            if s.channel_id == channel_id {
                s.handle.resize(col_width as u16, row_height as u16);
            }
        }

        Ok(())
    }

    /// Handle data from SSH client.
    async fn data(
        &mut self,
        channel_id: ChannelId,
        data: &[u8],
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        if let Some(ref mut s) = self.session {
            if s.channel_id == channel_id {
                // Parse input and send to TUI
                s.handle.process_input(data);
                // Note: Output is forwarded by the background task started in shell_request
            }
        }

        Ok(())
    }

    /// Handle channel close.
    async fn channel_close(
        &mut self,
        channel_id: ChannelId,
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        if let Some(ref s) = self.session {
            if s.channel_id == channel_id {
                log::info!("Channel closed for {:?}", self.peer_addr);
                self.session = None;
            }
        }

        Ok(())
    }

    /// Handle channel EOF.
    async fn channel_eof(
        &mut self,
        channel_id: ChannelId,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        log::debug!("Channel EOF on {:?}", channel_id);

        // Close the channel when we receive EOF
        if let Some(ref s) = self.session {
            if s.channel_id == channel_id {
                session.close(channel_id)?;
            }
        }

        Ok(())
    }
}

/// Simple handler that doesn't spawn a TUI application.
///
/// Useful for testing or when you want to manually manage the TUI lifecycle.
pub struct SimpleTuiHandler {
    session: Option<TuiSession>,
    peer_addr: Option<std::net::SocketAddr>,
}

impl SimpleTuiHandler {
    /// Create a new simple handler.
    pub fn new(peer_addr: Option<std::net::SocketAddr>) -> Self {
        Self {
            session: None,
            peer_addr,
        }
    }

    /// Get the current session, if any.
    pub fn session(&self) -> Option<&TuiSession> {
        self.session.as_ref()
    }

    /// Get the current session mutably, if any.
    pub fn session_mut(&mut self) -> Option<&mut TuiSession> {
        self.session.as_mut()
    }
}
