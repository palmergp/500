//! fh-host: hosts a game of 500 on your local network. Run it on one computer, then everyone at
//! the table opens the address it prints (or scans its QR code) on their phone or laptop, over
//! the same wifi. The host serves the web app, keeps the one true copy of the game and runs the
//! bots and the coach; browsers only ever see their own cards.
//!
//!     cargo run --release -p fh-host            (after `npm run build` in web/)
//!     fh-host --port 5500 --dist web/dist --state fh-host-table.json

mod room;

use std::net::{IpAddr, Ipv4Addr, UdpSocket};
use std::path::{Path, PathBuf};

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::header;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use futures_util::{SinkExt, StreamExt};
use qrcode::render::{svg, unicode};
use qrcode::QrCode;
use room::RoomHandle;
use tower_http::services::{ServeDir, ServeFile};

const USAGE: &str = "\
fh-host: host a game of 500 for friends on the same wifi

Usage: fh-host [--port N] [--dist DIR] [--state FILE | --fresh]

  --port N      port to listen on (default 5500)
  --dist DIR    the built web app (default: web/dist, found from here or next to the program)
  --state FILE  where the table is kept between runs (default fh-host-table.json)
  --fresh       start with an empty table, ignoring any saved one
";

struct Options {
    port: u16,
    dist: Option<PathBuf>,
    state: PathBuf,
    fresh: bool,
}

fn parse_args() -> Result<Options, String> {
    let mut options = Options { port: 5500, dist: None, state: PathBuf::from("fh-host-table.json"), fresh: false };
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = || args.next().ok_or_else(|| format!("{arg} needs a value"));
        match arg.as_str() {
            "--port" => options.port = value()?.parse().map_err(|_| "--port needs a number")?,
            "--dist" => options.dist = Some(PathBuf::from(value()?)),
            "--state" => options.state = PathBuf::from(value()?),
            "--fresh" => options.fresh = true,
            "-h" | "--help" => {
                print!("{USAGE}");
                std::process::exit(0);
            }
            other => return Err(format!("unknown option {other}\n\n{USAGE}")),
        }
    }
    Ok(options)
}

/// The built web app: the given directory, else web/dist from here or from the program's folder.
fn find_dist(given: Option<PathBuf>) -> Result<PathBuf, String> {
    let exe = std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf));
    let candidates: Vec<PathBuf> = match given {
        Some(dir) => vec![dir],
        None => {
            let mut list = vec![PathBuf::from("web/dist")];
            if let Some(exe) = exe {
                // Next to the program (a copied install), or from target/release back to the repo.
                list.extend([exe.join("web"), exe.join("../../web/dist")]);
            }
            list
        }
    };
    candidates.iter().find(|d| d.join("index.html").is_file()).cloned().ok_or_else(|| {
        format!(
            "can't find the built web app (looked in {}).\nBuild it first: cd web && npm run build",
            candidates.iter().map(|d| d.display().to_string()).collect::<Vec<_>>().join(", ")
        )
    })
}

/// This computer's address on the local network. Connecting a UDP socket sends nothing; it only
/// asks the OS which interface would be used.
fn local_ip() -> IpAddr {
    // The interface with the default route; without one (wifi with no internet), a home range.
    [Ipv4Addr::new(8, 8, 8, 8), Ipv4Addr::new(192, 168, 0, 1)]
        .into_iter()
        .find_map(|target| {
            let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).ok()?;
            socket.connect((target, 9)).ok()?;
            Some(socket.local_addr().ok()?.ip())
        })
        .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST))
}

#[derive(Clone)]
struct AppState {
    room: RoomHandle,
    url: String,
}

#[tokio::main]
async fn main() {
    let options = parse_args().unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(2);
    });
    let dist = find_dist(options.dist).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1);
    });
    if options.fresh {
        let _ = std::fs::remove_file(&options.state);
    }
    let (room, resumed) = room::spawn(Some(options.state.clone()));
    let url = format!("http://{}:{}/", local_ip(), options.port);
    let state = AppState { room, url: url.clone() };

    let app = Router::new()
        .route("/api/info", get(info))
        .route("/api/qr.svg", get(qr_svg))
        .route("/ws", get(upgrade))
        .fallback_service(ServeDir::new(&dist).fallback(ServeFile::new(dist.join("index.html"))))
        .with_state(state);

    let listener = match tokio::net::TcpListener::bind((Ipv4Addr::UNSPECIFIED, options.port)).await {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("can't listen on port {}: {e}\nIs the host already running? Try --port 5501.", options.port);
            std::process::exit(1);
        }
    };
    println!("500 is hosting a table on your network.\n");
    if let Ok(code) = QrCode::new(url.as_bytes()) {
        println!("{}", code.render::<unicode::Dense1x2>().quiet_zone(true).build());
    }
    println!("  On this computer:   http://localhost:{}/", options.port);
    println!("  Everyone else:      {url}   (same wifi; scan the code above)\n");
    match resumed {
        Some(game) => println!("Picked up {game} from {}.", options.state.display()),
        None => println!("Serving the app from {}.", dist.display()),
    }
    println!("If friends can't connect, allow fh-host through the firewall for private networks.");
    println!("Press Ctrl+C to stop; the table is kept for next time.\n");

    let shutdown = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    if let Err(e) = axum::serve(listener, app).with_graceful_shutdown(shutdown).await {
        eprintln!("server error: {e}");
    }
}

/// Lets the web app know it was loaded from a host, and the address to share.
async fn info(State(state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({ "app": "fh-host", "version": env!("CARGO_PKG_VERSION"), "url": state.url }))
}

/// The join address as a QR code, for the lobby screen.
async fn qr_svg(State(state): State<AppState>) -> Response {
    let Ok(code) = QrCode::new(state.url.as_bytes()) else { return "no code".into_response() };
    let image = code.render::<svg::Color>().min_dimensions(200, 200).quiet_zone(true).build();
    ([(header::CONTENT_TYPE, "image/svg+xml"), (header::CACHE_CONTROL, "no-store")], image).into_response()
}

async fn upgrade(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| connection(socket, state.room))
}

/// One browser: messages in go to the room; the room's messages go out. Either side ending (the
/// browser leaving, or the room dropping a silent connection) closes the socket.
async fn connection(socket: WebSocket, room: RoomHandle) {
    let (mut sink, mut stream) = socket.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    let conn = room.connect(tx);
    let writer = async {
        while let Some(text) = rx.recv().await {
            if sink.send(Message::Text(text.into())).await.is_err() {
                break;
            }
        }
        let _ = sink.close().await;
    };
    let reader = async {
        while let Some(Ok(message)) = stream.next().await {
            match message {
                Message::Text(text) => room.message(conn, text.to_string()),
                Message::Close(_) => break,
                _ => {}
            }
        }
    };
    tokio::select! {
        _ = writer => {}
        _ = reader => {}
    }
    room.disconnect(conn);
}
