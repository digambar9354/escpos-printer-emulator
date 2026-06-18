use crate::emulator::EmulatorState;
use crate::escpos::parser::EscPosParser;
use anyhow::Result;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tracing::{info, warn, error};

pub async fn start_server(emulator_state: Arc<Mutex<EmulatorState>>) -> Result<()> {
    // Bind to 0.0.0.0 so other devices on the LAN (e.g. a phone) can connect, not just localhost
    let listener = TcpListener::bind("0.0.0.0:9100").await?;
    info!("ESC/POS Emulator server listening on 0.0.0.0:9100 (reachable from the local network)");

    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                info!("New connection from: {}", addr);
                let state = emulator_state.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(socket, state).await {
                        error!("Error handling connection from {}: {}", addr, e);
                    }
                });
            }
            Err(e) => {
                error!("Failed to accept connection: {}", e);
            }
        }
    }
}

async fn handle_connection(
    mut socket: TcpStream,
    emulator_state: Arc<Mutex<EmulatorState>>,
) -> Result<()> {
    // The parser keeps its own buffer across reads, so feed it raw chunks directly.
    let mut parser = EscPosParser::new();
    let mut chunk = vec![0u8; 4096];

    loop {
        match socket.read(&mut chunk).await {
            Ok(0) => {
                info!("Connection closed by client");
                break;
            }
            Ok(n) => {
                let data = &chunk[..n];

                // Answer real-time status queries (DLE EOT n) immediately so POS apps
                // see the printer as online and don't retry/duplicate the job.
                if let Some(resp) = status_responses(data) {
                    if let Err(e) = socket.write_all(&resp).await {
                        warn!("Failed to send status response: {}", e);
                    }
                }

                if let Ok(commands) = parser.parse_stream(data) {
                    let mut state = emulator_state.lock().await;
                    for command in commands {
                        state.process_command(&command);
                    }
                }
            }
            Err(e) => {
                warn!("Error reading from socket: {}", e);
                break;
            }
        }
    }

    Ok(())
}

/// Build responses for any ESC/POS real-time status requests (`DLE EOT n`) found in `data`.
/// Returns "online, paper present, no error" status bytes so POS apps proceed normally.
fn status_responses(data: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut i = 0;
    while i + 2 < data.len() {
        if data[i] == 0x10 && data[i + 1] == 0x04 {
            let n = data[i + 2];
            // n=1 printer status -> 0x16 (online); others -> 0x12 (no error / paper present)
            out.push(if n == 1 { 0x16 } else { 0x12 });
            i += 3;
        } else {
            i += 1;
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}
