mod handler;
mod request;
mod response;
mod room;

use crate::handler::Handler;
use argh::FromArgs;
use futures_channel::mpsc::unbounded;
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};
use log::{debug, info};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{io, io::Error as IoError, net::SocketAddr};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{interval, Duration};
use tokio_rustls::rustls::internal::pemfile::{certs, pkcs8_private_keys};
use tokio_rustls::rustls::{Certificate, NoClientAuth, PrivateKey, ServerConfig};
use tokio_rustls::server::TlsStream;
use tokio_rustls::TlsAcceptor;

const CLEANER_INTERVAL: u64 = 5 * 60;

/// Beacon server
#[derive(Debug, FromArgs)]
struct Options {
    /// bind addr
    #[argh(positional, default = "String::from(\"127.0.0.1:3030\")")]
    addr: String,

    /// cert file
    #[argh(option, short = 'c')]
    cert: PathBuf,

    /// key file
    #[argh(option, short = 'k')]
    key: PathBuf,
}

async fn register_cleaner(handler: Handler) {
    let mut interval = interval(Duration::from_secs(CLEANER_INTERVAL));
    loop {
        interval.tick().await;
        handler.clean();
    }
}

async fn handle_connection(handler: Handler, raw_stream: TlsStream<TcpStream>, addr: SocketAddr) {
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

fn load_certs(path: &Path) -> io::Result<Vec<Certificate>> {
    certs(&mut BufReader::new(File::open(path)?))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid cert"))
}

fn load_keys(path: &Path) -> io::Result<Vec<PrivateKey>> {
    pkcs8_private_keys(&mut BufReader::new(File::open(path)?))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid key"))
}

#[tokio::main]
async fn main() -> Result<(), IoError> {
    let _ = env_logger::try_init();
    let options: Options = argh::from_env();

    let handler = Handler::new();

    // Create a cleaning routine
    tokio::spawn(register_cleaner(handler.clone()));

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&options.addr).await;
    let listener = try_socket.expect("Failed to bind");
    info!("Listening on: {}", &options.addr);

    let certs = load_certs(&options.cert)?;
    let mut keys = load_keys(&options.key)?;

    let mut config = ServerConfig::new(NoClientAuth::new());
    config
        .set_single_cert(certs, keys.remove(0))
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;
    let acceptor = TlsAcceptor::from(Arc::new(config));

    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        let acceptor = acceptor.clone();
        if let Ok(stream) = acceptor.accept(stream).await {
            tokio::spawn(handle_connection(handler.clone(), stream, addr));
        }
    }

    Ok(())
}
