// (C) 2026 - Enzo Lombardi

//! Remote keyboard input over TCP (disabled by default).
//!
//! This is a testing/automation aid: when enabled, turbo-vision listens on a
//! local TCP port and converts incoming text lines into keyboard events that
//! are injected into the application's event loop, exactly as if the keys had
//! been pressed.
//!
//! Each line may contain one or more whitespace-separated key chords; the
//! chords are parsed by [`crate::core::event::parse_key_chord`]
//! and queued in order. For example, sending the line:
//!
//! ```text
//! CTRL+F12 ALT+X
//! ```
//!
//! injects a `Ctrl+F12` press (e.g. take a screenshot) followed by `Alt+X`.
//!
//! The listener binds to `127.0.0.1` only, so it is reachable from the local
//! machine but not from the network. It is **off by default** and must be
//! enabled explicitly via
//! [`Terminal::enable_remote_input`](crate::terminal::Terminal::enable_remote_input)
//! or [`Application::enable_remote_input`](crate::app::Application::enable_remote_input).
//!
//! # Example
//!
//! ```bash
//! # With remote input enabled on port 8888:
//! printf 'CTRL+F12\n'   | nc 127.0.0.1 8888   # key chord(s)
//! printf 'CLICK 28 23\n' | nc 127.0.0.1 8888   # left-click at cell (28,23)
//! ```
//!
//! Mouse lines use `CLICK x y` (left button), `RCLICK x y`, or `MCLICK x y`
//! with 0-indexed cell coordinates; any other line is parsed as key chords.

use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::sync::mpsc::Sender;
use std::thread;

use crate::core::event::{
    Event, EventType, MB_LEFT_BUTTON, MB_MIDDLE_BUTTON, MB_RIGHT_BUTTON, parse_key_chord,
};
use crate::core::geometry::Point;

/// Parse one protocol line into the events it should inject.
///
/// Two line forms are recognized:
/// * Mouse: `CLICK x y` (left button), `RCLICK x y`, or `MCLICK x y` — emits a
///   button-down then button-up at the given 0-indexed cell coordinates.
/// * Keys: anything else is treated as whitespace-separated key chords (see
///   [`parse_key_chord`]), e.g. `CTRL+F12 ALT+X`.
///
/// Returns an empty vector for a blank or unparseable line.
pub fn parse_line(line: &str) -> Vec<Event> {
    let mut tokens = line.split_whitespace();
    let Some(first) = tokens.next() else {
        return Vec::new();
    };

    let button = match first.to_ascii_uppercase().as_str() {
        "CLICK" | "LCLICK" => Some(MB_LEFT_BUTTON),
        "RCLICK" => Some(MB_RIGHT_BUTTON),
        "MCLICK" => Some(MB_MIDDLE_BUTTON),
        _ => None,
    };

    if let Some(button) = button {
        let coords: Vec<&str> = tokens.collect();
        if let [xs, ys] = coords[..] {
            if let (Ok(x), Ok(y)) = (xs.parse::<i16>(), ys.parse::<i16>()) {
                let pos = Point::new(x, y);
                return vec![
                    Event::mouse(EventType::MouseDown, pos, button, false),
                    Event::mouse(EventType::MouseUp, pos, 0, false),
                ];
            }
        }
        return Vec::new(); // malformed mouse line
    }

    // Otherwise: one or more key chords.
    line.split_whitespace()
        .filter_map(parse_key_chord)
        .collect()
}

/// Bind a TCP listener on `127.0.0.1:port` and forward parsed key events to `tx`.
///
/// The listener and per-connection handlers run on detached background threads,
/// so this returns as soon as the socket is bound. Each accepted connection is
/// read line by line; every whitespace-separated chord on a line is parsed and,
/// if valid, sent through `tx`. Unparseable chords are logged and skipped.
///
/// # Errors
///
/// Returns an error if the port cannot be bound (e.g. already in use).
pub fn spawn(port: u16, tx: Sender<Event>) -> std::io::Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    log::info!("Remote key input listening on 127.0.0.1:{port}");

    thread::spawn(move || {
        for stream in listener.incoming() {
            let stream = match stream {
                Ok(s) => s,
                Err(e) => {
                    log::warn!("Remote key input: accept failed: {e}");
                    continue;
                }
            };
            let tx = tx.clone();
            thread::spawn(move || {
                let reader = BufReader::new(stream);
                for line in reader.lines() {
                    let Ok(line) = line else { break };
                    let events = parse_line(&line);
                    if events.is_empty() && !line.trim().is_empty() {
                        log::warn!("Remote input: unparseable line {line:?}");
                    }
                    for event in events {
                        // If the receiver is gone the app has exited; stop.
                        if tx.send(event).is_err() {
                            return;
                        }
                    }
                }
            });
        }
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::event::{KB_ALT_X, KB_CTRL_F12};

    #[test]
    fn parses_click_into_down_up() {
        let evs = parse_line("CLICK 28 23");
        assert_eq!(evs.len(), 2);
        assert_eq!(evs[0].what, EventType::MouseDown);
        assert_eq!(evs[0].mouse.pos, Point::new(28, 23));
        assert_eq!(evs[0].mouse.buttons, MB_LEFT_BUTTON);
        assert_eq!(evs[1].what, EventType::MouseUp);
    }

    #[test]
    fn parses_key_chords_line() {
        let evs = parse_line("CTRL+F12 ALT+X");
        assert_eq!(evs.len(), 2);
        assert_eq!(evs[0].key_code, KB_CTRL_F12);
        assert_eq!(evs[1].key_code, KB_ALT_X);
    }

    #[test]
    fn blank_and_malformed_yield_nothing() {
        assert!(parse_line("").is_empty());
        assert!(parse_line("   ").is_empty());
        assert!(parse_line("CLICK 1").is_empty());
        assert!(parse_line("CLICK x y").is_empty());
    }
}
