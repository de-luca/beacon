mod handler;
mod request;
mod response;
mod room;

use crate::handler::Handler;
use argh::FromArgs;
use futures_util::{SinkExt, StreamExt, TryFutureExt};
use log::{debug, error, info};
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::WebSocket;
use warp::Filter;

const CLEANER_INTERVAL: u64 = 60;

/// Beacon server
#[derive(Debug, FromArgs)]
struct Options {
    /// bind addr
    #[argh(positional, default = "String::from(\"127.0.0.1:3030\")")]
    addr: String,

    /// cert file
    #[argh(option, short = 'c')]
    cert: Option<PathBuf>,

    /// key file
    #[argh(option, short = 'k')]
    key: Option<PathBuf>,
}

async fn register_cleaner(handler: Handler) {
    let mut interval = interval(Duration::from_secs(CLEANER_INTERVAL));
    loop {
        interval.tick().await;
        debug!("Checking rooms");
        handler.clean();
    }
}

async fn handle_connection(ws: WebSocket, handler: Handler) {
    let (mut ws_tx, mut ws_rx) = ws.split();
    let (tx, rx) = mpsc::unbounded_channel();
    let mut rx = UnboundedReceiverStream::new(rx);

    let peer = handler.add_peer(tx);

    debug!("New Peer: {}", &peer);

    tokio::task::spawn(async move {
        while let Some(message) = rx.next().await {
            ws_tx
                .send(message)
                .unwrap_or_else(|e| error!("websocket send error: {}", e))
                .await;
        }
    });

    while let Some(result) = ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                error!("Websocket error: {}", e);
                break;
            }
        };
        handler.handle(peer, msg);
    }

    debug!("Removed Peer: {}", &peer);
    handler.remove_peer(&peer);
}

#[tokio::main]
async fn main() {
    let _ = env_logger::try_init();
    let options: Options = argh::from_env();

    let handler = Handler::new();

    tokio::spawn(register_cleaner(handler.clone()));
    info!("Registered room cleaner");

    let handler = warp::any().map(move || handler.clone());

    let addr: SocketAddr = options.addr.parse().expect("Cannot parse addr!");

    let beacon = warp::ws()
        .and(handler.clone())
        .map(|ws: warp::ws::Ws, handler| ws.on_upgrade(move |ws| handle_connection(ws, handler)));

    let stats = warp::get()
        .and(handler.clone())
        .map(|handler: Handler| warp::reply::json(&handler.get_stats()));

    let routes = beacon.or(stats);

    if options.cert.is_some() && options.key.is_some() {
        info!("Starting with TLS");
        warp::serve(routes)
            .tls()
            .cert_path(options.cert.as_ref().unwrap())
            .key_path(options.key.as_ref().unwrap())
            .run(addr)
            .await;
    } else {
        info!("Starting without TLS");
        warp::serve(routes).run(addr).await;
    }
}
