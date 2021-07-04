mod handler;
mod request;
mod response;
mod room;

use crate::handler::Handler;
use futures_channel::mpsc::unbounded;
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};
use log::{debug, info};
use std::{env, io::Error as IoError, net::SocketAddr};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{interval, Duration};

const CLEANER_INTERVAL: u64 = 5 * 60;

async fn register_cleaner(handler: Handler) {
    let mut interval = interval(Duration::from_secs(CLEANER_INTERVAL));
    loop {
        interval.tick().await;
        handler.clean();
    }
}

async fn handle_connection(handler: Handler, raw_stream: TcpStream, addr: SocketAddr) {
    debug!("Incoming TCP connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    debug!("WebSocket connection established: {}", addr);

    // Insert the write part of this peer to the peer map.
    let (tx, rx) = unbounded();
    let peer_id = handler.add_peer(tx);

    let (outgoing, incoming) = ws_stream.split();

    let in_handler = incoming.try_for_each(|msg| {
        debug!(
            "Received a message from {}: {}",
            addr,
            msg.to_text().unwrap()
        );
        handler.handle(peer_id, msg);
        future::ok(())
    });

    let out_handler = rx.map(Ok).forward(outgoing);

    pin_mut!(in_handler, out_handler);
    future::select(in_handler, out_handler).await;

    debug!("{} disconnected", &addr);
    handler.remove_peer(&peer_id);
}

#[tokio::main]
async fn main() -> Result<(), IoError> {
    let _ = env_logger::try_init();
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:3030".to_string());

    let handler = Handler::new();

    // Create a cleaning routine
    tokio::spawn(register_cleaner(handler.clone()));

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    info!("Listening on: {}", addr);

    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(handler.clone(), stream, addr));
    }

    Ok(())
}
