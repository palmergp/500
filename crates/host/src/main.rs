//! fh-host: runs the game of 500 on this computer, and hosts it for friends on the same wifi.
//! It serves the web app, keeps the one true copy of the game and runs the bots and the coach;
//! browsers only ever see their own cards. It opens the game in your browser when it starts.
//!
//!     cargo run --release -p fh-host            (after `npm run build` in web/)
//!     fh-host --port 5500 --dist web/dist --state fh-host-table.json
//!
//! Built with `--features embed`, the web app is inside the program, so the one file runs on any
//! Windows PC (`make-app.cmd` builds it as `app/500.exe`).

mod room;

use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream, UdpSocket};
use std::path::{Path, PathBuf};
use std::time::Duration;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::header;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use futures_util::{SinkExt, StreamExt};
use qrcode::render::{svg, unicode};
use qrcode::QrCode;
use room::RoomHandle;
use tower_http::services::{ServeDir, ServeFile};

const USAGE: &str = "\
fh-host: play 500 here, and host it for friends on the same wifi

Usage: fh-host [--port N] [--dist DIR] [--state FILE | --fresh] [--no-browser]

  --port N      port to listen on (default 5500)
  --dist DIR    the built web app (default: built in, or web/dist found from here)
  --state FILE  where the table is kept between runs
  --fresh       start with an empty table, ignoring any saved one
  --no-browser  don't open the game in the browser
";

const DEFAULT_PORT: u16 = 5500;

/// The web app built into the program.
#[cfg(feature = "embed")]
static EMBEDDED: include_dir::Dir<'static> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/../../web/dist");

struct Options {
    port: Option<u16>,
    dist: Option<PathBuf>,
    state: Option<PathBuf>,
    fresh: bool,
    browser: bool,
}

fn parse_args() -> Result<Options, String> {
    let mut options = Options { port: None, dist: None, state: None, fresh: false, browser: true };
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = || args.next().ok_or_else(|| format!("{arg} needs a value"));
        match arg.as_str() {
            "--port" => options.port = Some(value()?.parse().map_err(|_| "--port needs a number")?),
            "--dist" => options.dist = Some(PathBuf::from(value()?)),
            "--state" => options.state = Some(PathBuf::from(value()?)),
            "--fresh" => options.fresh = true,
            "--no-browser" => options.browser = false,
            "-h" | "--help" => {
                print!("{USAGE}");
                std::process::exit(0);
            }
            other => return Err(format!("unknown option {other}\n\n{USAGE}")),
        }
    }
    Ok(options)
}

/// Says what went wrong and stops. A program started by double-clicking closes its window at
/// once, so the built-in version waits for Enter first.
fn fail(message: &str) -> ! {
    eprintln!("\n{message}");
    if cfg!(feature = "embed") {
        eprintln!("\nPress Enter to close this window.");
        let _ = std::io::stdin().read_line(&mut String::new());
    }
    std::process::exit(1);
}

/// Where the web app comes from: a folder on disk, or the copy built into the program.
enum Files {
    Disk(PathBuf),
    #[cfg(feature = "embed")]
    Embedded,
}

impl Files {
    /// The folder the app is served from, if it isn't built in.
    fn folder(&self) -> Option<&Path> {
        match self {
            Files::Disk(dist) => Some(dist),
            #[cfg(feature = "embed")]
            Files::Embedded => None,
        }
    }
}

fn find_files(given: Option<PathBuf>) -> Result<Files, String> {
    #[cfg(feature = "embed")]
    if given.is_none() {
        return Ok(Files::Embedded);
    }
    find_dist(given).map(Files::Disk)
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

/// Where the table is kept between runs: next to the project for a developer's copy, in the
/// user's own app-data folder for the single-file program (which may sit anywhere, even a
/// read-only download folder).
fn default_state() -> PathBuf {
    if cfg!(feature = "embed") {
        if let Some(base) = std::env::var_os("LOCALAPPDATA").or_else(|| std::env::var_os("HOME")) {
            let dir = PathBuf::from(base).join("500");
            if std::fs::create_dir_all(&dir).is_ok() {
                return dir.join("table.json");
            }
        }
    }
    PathBuf::from("fh-host-table.json")
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

/// Whether this game is already running on `port` (someone double-clicked it twice).
fn already_running(port: u16) -> bool {
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    let Ok(mut stream) = TcpStream::connect_timeout(&address, Duration::from_secs(1)) else { return false };
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    if stream.write_all(b"GET /api/info HTTP/1.0\r\nHost: localhost\r\n\r\n").is_err() {
        return false;
    }
    let mut reply = String::new();
    let _ = stream.read_to_string(&mut reply);
    reply.contains("\"fh-host\"")
}

/// Opens `url` in the default browser.
fn open_browser(url: &str) {
    let result = if cfg!(windows) {
        // The empty "" is start's window title, so the URL isn't taken for one.
        std::process::Command::new("cmd").args(["/C", "start", "", url]).spawn()
    } else if cfg!(target_os = "macos") {
        std::process::Command::new("open").arg(url).spawn()
    } else {
        std::process::Command::new("xdg-open").arg(url).spawn()
    };
    if result.is_err() {
        println!("Open {url} in your browser to play.");
    }
}

#[derive(Clone)]
struct AppState {
    room: RoomHandle,
    url: String,
}

#[tokio::main]
async fn main() {
    let options = parse_args().unwrap_or_else(|e| fail(&e));
    let files = find_files(options.dist).unwrap_or_else(|e| fail(&e));

    // Port 5500 unless told otherwise. If the game is already running there, just show it. If
    // something else has the port, try the next few (a chosen port is used or nothing).
    let ports: Vec<u16> = match options.port {
        Some(port) => vec![port],
        None => (DEFAULT_PORT..DEFAULT_PORT + 10).collect(),
    };
    let mut bound = None;
    for &port in &ports {
        if already_running(port) {
            let local = format!("http://localhost:{port}/");
            if options.browser {
                println!("500 is already running: opening it in your browser.");
                open_browser(&local);
            } else {
                println!("500 is already running at {local}");
            }
            std::thread::sleep(Duration::from_secs(2));
            return;
        }
        if let Ok(listener) = tokio::net::TcpListener::bind((Ipv4Addr::UNSPECIFIED, port)).await {
            bound = Some((listener, port));
            break;
        }
    }
    let Some((listener, port)) = bound else {
        fail(&format!("Can't start: port {} is in use by another program. Try --port 5600.", ports[0]))
    };

    let state_path = options.state.unwrap_or_else(default_state);
    if options.fresh {
        let _ = std::fs::remove_file(&state_path);
    }
    let (room, resumed) = room::spawn(Some(state_path.clone()));
    let url = format!("http://{}:{}/", local_ip(), port);
    let local = format!("http://localhost:{port}/");
    let state = AppState { room, url: url.clone() };

    let app = Router::new()
        .route("/api/info", get(info))
        .route("/api/qr.svg", get(qr_svg))
        .route("/api/hands", post(save_hand))
        .route("/ws", get(upgrade));
    let app = match &files {
        Files::Disk(dist) => app.fallback_service(ServeDir::new(dist).fallback(ServeFile::new(dist.join("index.html")))),
        #[cfg(feature = "embed")]
        Files::Embedded => app.fallback(embedded),
    };
    let app = app.with_state(state);

    println!("500 is running.\n");
    if options.browser {
        println!("  Your browser should open the game. If it doesn't, open {local}");
    } else {
        println!("  Open {local} in your browser to play.");
    }
    println!("  Keep this window open while you play, and close it when you're done.");
    if let Some(game) = &resumed {
        println!("  Your table has {game}.");
    }
    println!("\nTo play with friends on the same wifi, they open {url}");
    println!("on their phone or laptop, or scan this code:");
    if let Ok(code) = QrCode::new(url.as_bytes()) {
        println!("{}", code.render::<unicode::Dense1x2>().quiet_zone(true).build());
    }
    if let Some(dist) = files.folder() {
        println!("Serving the app from {}.", dist.display());
    }
    println!("If friends can't connect, allow this program through the Windows firewall for private networks.");
    if options.browser {
        open_browser(&local);
    }

    let shutdown = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    if let Err(e) = axum::serve(listener, app).with_graceful_shutdown(shutdown).await {
        eprintln!("server error: {e}");
    }
}

/// Serves the built-in web app. Paths without a file extension get the app itself.
#[cfg(feature = "embed")]
async fn embedded(uri: axum::http::Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };
    let file = EMBEDDED.get_file(path).or_else(|| if path.contains('.') { None } else { EMBEDDED.get_file("index.html") });
    let Some(file) = file else { return axum::http::StatusCode::NOT_FOUND.into_response() };
    let mime = mime_guess::from_path(file.path()).first_or_octet_stream().to_string();
    // Built assets have content hashes in their names, so they never change.
    let cache = if path.starts_with("assets/") { "public, max-age=31536000, immutable" } else { "no-cache" };
    ([(header::CONTENT_TYPE, mime), (header::CACHE_CONTROL, cache.to_string())], file.contents()).into_response()
}

/// A hand code from the web app, with the game it names. Kept in the user's app-data folder by
/// every copy of the host (`fh_wasm::table::hands_dir`), so `fh-lab show --code ...` finds it.
#[derive(serde::Deserialize)]
struct HandCode {
    code: String,
    /// The game up to the moment the code names (a save, as JSON).
    save: String,
}

/// Keeps a hand code's game so the code can be looked up later (a question about a hand).
async fn save_hand(Json(hand): Json<HandCode>) -> Response {
    let ok_code = !hand.code.is_empty() && hand.code.len() <= 16 && hand.code.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
    if !ok_code || hand.save.len() > 200_000 || serde_json::from_str::<fh_wasm::table::SaveData>(&hand.save).is_err() {
        return axum::http::StatusCode::BAD_REQUEST.into_response();
    }
    let Some(dir) = fh_wasm::table::hands_dir() else { return axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response() };
    let record = serde_json::json!({ "code": hand.code, "save": hand.save });
    match std::fs::create_dir_all(&dir).and_then(|_| std::fs::write(dir.join(format!("{}.json", hand.code)), record.to_string())) {
        Ok(()) => Json(serde_json::json!({ "ok": true })).into_response(),
        Err(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response(),
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
