use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use slint::Weak;
use std::{fs, io::Write, net::IpAddr, path::{Path, PathBuf}, sync::{Arc, atomic::{AtomicBool, Ordering}}, time::{Duration, Instant}};
use tokio::{sync::mpsc, time};
use tokio_tungstenite::{connect_async, tungstenite::{client::IntoClientRequest, http::HeaderValue, Message}};

use crate::MainWindow;

#[derive(Clone, Serialize, Deserialize)]
struct Session {
    base_url: String,
    token: String,
}

enum Command {
    Pair { host: String, code: String },
    Retry,
    Forget,
    Input(Value),
}

pub fn attach(ui: &MainWindow, storage: Option<PathBuf>) {
    let (tx, rx) = mpsc::unbounded_channel();
    let ready = Arc::new(AtomicBool::new(false));
    {
        let tx = tx.clone();
        ui.on_pair_request(move |host, code| {
            let _ = tx.send(Command::Pair { host: host.to_string(), code: code.to_string() });
        });
    }
    {
        let tx = tx.clone();
        ui.on_reconnect_request(move || { let _ = tx.send(Command::Retry); });
    }
    {
        let tx = tx.clone();
        ui.on_forget_request(move || { let _ = tx.send(Command::Forget); });
    }
    {
        let tx = tx.clone();
        let ready = ready.clone();
        ui.on_pointer_move(move |dx, dy| {
            if !ready.load(Ordering::Relaxed) { return; }
            let dx = (dx * 1.7).round().clamp(-250.0, 250.0) as i32;
            let dy = (dy * 1.7).round().clamp(-250.0, 250.0) as i32;
            if dx != 0 || dy != 0 { let _ = tx.send(Command::Input(json!({"type":"move","dx":dx,"dy":dy}))); }
        });
    }
    {
        let tx = tx.clone();
        let ready = ready.clone();
        ui.on_remote_click(move |button| {
            if ready.load(Ordering::Relaxed) && ["left", "right", "middle"].contains(&button.as_str()) {
                let _ = tx.send(Command::Input(json!({"type":"click","button":button.as_str()})));
            }
        });
    }
    {
        let tx = tx.clone();
        let ready = ready.clone();
        ui.on_remote_scroll(move |amount| {
            if ready.load(Ordering::Relaxed) && amount != 0 {
                let _ = tx.send(Command::Input(json!({"type":"scroll","amount":amount.clamp(-8,8)})));
            }
        });
    }
    {
        let tx = tx.clone();
        let ready = ready.clone();
        ui.on_remote_drag(move |down| {
            if ready.load(Ordering::Relaxed) {
                let _ = tx.send(Command::Input(json!({"type":"drag","down":down})));
            }
        });
    }
    {
        let tx = tx.clone();
        let ready = ready.clone();
        ui.on_send_key(move |name| {
            if ready.load(Ordering::Relaxed) &&
                ["enter","backspace","escape","tab","space","up","down","left","right","home","f11"].contains(&name.as_str()) {
                let _ = tx.send(Command::Input(json!({"type":"key","name":name.as_str()})));
            }
        });
    }
    {
        let tx = tx.clone();
        let ready = ready.clone();
        ui.on_send_text(move |text| {
            if !ready.load(Ordering::Relaxed) { return; }
            // The existing Pi input emulator supports US ASCII. Use clipboard for Unicode.
            let text: String = text.chars().filter(char::is_ascii).collect();
            for segment in text.as_bytes().chunks(200) {
                let segment = String::from_utf8_lossy(segment).into_owned();
                if !segment.is_empty() { let _ = tx.send(Command::Input(json!({"type":"text","value":segment}))); }
            }
        });
    }
    {
        let tx = tx.clone();
        let ready = ready.clone();
        ui.on_clipboard_paste(move |text| {
            let text = text.to_string();
            if ready.load(Ordering::Relaxed) && !text.is_empty() && text.chars().count() <= 4000 && text.len() <= 12000 {
                let _ = tx.send(Command::Input(json!({"type":"clipboard-set","value":text,"action":"paste"})));
            }
        });
    }
    {
        let tx = tx.clone();
        let ready = ready.clone();
        ui.on_launch_moonlight(move || {
            if ready.load(Ordering::Relaxed) {
                let _ = tx.send(Command::Input(json!({"type":"app-launch","id":"moonlight"})));
            }
        });
    }
    let weak = ui.as_weak();
    std::thread::spawn(move || {
        if let Ok(rt) = tokio::runtime::Builder::new_current_thread().enable_all().build() {
            rt.block_on(network_worker(rx, weak, ready, storage));
        }
    });
}

fn update(weak: &Weak<MainWindow>, state: &str, detail: &str, connected: bool) {
    let state = state.to_owned();
    let detail = detail.to_owned();
    let _ = weak.upgrade_in_event_loop(move |ui| {
        ui.set_connection_state(state.into());
        ui.set_status_detail(detail.into());
        ui.set_is_connected(connected);
    });
}

fn host_base(host: &str) -> Result<String, &'static str> {
    let host = host.trim().trim_end_matches('/');
    if host.len() > 160 || host.is_empty() { return Err("Enter a Pi hostname or LAN IP"); }
    let raw = if host.starts_with("http://") || host.starts_with("https://") {
        host.to_owned()
    } else {
        format!("http://{host}")
    };
    let url = reqwest::Url::parse(&raw).map_err(|_| "Invalid address")?;
    if !url.username().is_empty() || url.password().is_some() || url.query().is_some()
        || url.fragment().is_some() || url.path() != "/" {
        return Err("Enter only the Pi host and port, without a path or login");
    }
    let hostname = url.host_str().ok_or("Missing hostname")?;
    let local = hostname == "localhost" || hostname.ends_with(".local")
        || hostname.parse::<IpAddr>().map(|ip| match ip {
            IpAddr::V4(v) => v.is_private() || v.is_loopback() || v.is_link_local(),
            IpAddr::V6(v) => v.is_loopback() || v.is_unique_local() || v.is_unicast_link_local(),
        }).unwrap_or(false);
    if !local { return Err("Only local Pi addresses are supported"); }
    Ok(raw)
}

fn save_session(path: Option<&Path>, data: &Session) -> Result<(), String> {
    let Some(path) = path else { return Ok(()); };
    if let Some(parent) = path.parent() { fs::create_dir_all(parent).map_err(|_| "Cannot create settings directory")?; }
    let tmp = path.with_extension("tmp");
    let mut opts = fs::OpenOptions::new();
    opts.write(true).create(true).truncate(true);
    #[cfg(unix)]
    { use std::os::unix::fs::OpenOptionsExt; opts.mode(0o600); }
    let mut file = opts.open(&tmp).map_err(|_| "Cannot save pairing")?;
    file.write_all(&serde_json::to_vec(data).map_err(|_| "Cannot encode settings")?)
        .map_err(|_| "Cannot save pairing")?;
    file.sync_all().map_err(|_| "Cannot sync pairing")?;
    fs::rename(&tmp, path).map_err(|_| "Cannot store pairing")?;
    Ok(())
}

fn load_session(path: Option<&Path>) -> Option<Session> {
    let path = path?;
    let data: Session = serde_json::from_slice(&fs::read(path).ok()?).ok()?;
    if data.token.len() < 30 || host_base(&data.base_url).is_err() { return None; }
    Some(data)
}

fn forget_session(path: Option<&Path>) {
    if let Some(path) = path { let _ = fs::remove_file(path); }
}

async fn pair(client: &reqwest::Client, host: String, code: String) -> Result<Session, String> {
    if code.len() != 8 || !code.bytes().all(|c| c.is_ascii_digit()) {
        return Err("Pairing requires the eight-digit code shown on your Pi".into());
    }
    let base_url = host_base(&host).map_err(str::to_owned)?;
    let response = client.post(format!("{base_url}/api/pair"))
        .json(&json!({"code":code,"name":"Remora Android"})).send().await
        .map_err(|_| "Cannot reach Pi — check Wi-Fi and address".to_string())?;
    let status = response.status();
    let data: Value = response.json().await.map_err(|_| "Unexpected pairing response".to_string())?;
    if !status.is_success() {
        return Err(data["error"].as_str().unwrap_or("Pairing refused by Pi").to_owned());
    }
    let token = data["token"].as_str().ok_or("Pairing response missing token")?.to_owned();
    if token.len() < 30 { return Err("Invalid pairing token received".into()); }
    Ok(Session { base_url, token })
}

async fn verify(client: &reqwest::Client, session: &Session) -> Result<bool, String> {
    let response = client.post(format!("{}/api/verify",session.base_url))
        .json(&json!({"token":session.token})).send().await
        .map_err(|_| "Pi unreachable".to_string())?;
    if response.status() == reqwest::StatusCode::UNAUTHORIZED { return Ok(false); }
    if !response.status().is_success() { return Err("Pi verification failed".into()); }
    Ok(true)
}

fn websocket_address(base: &str) -> String {
    if let Some(rest) = base.strip_prefix("https://") { format!("wss://{rest}/ws") }
    else { format!("ws://{}/ws", base.trim_start_matches("http://")) }
}

async fn network_worker(
    mut rx: mpsc::UnboundedReceiver<Command>,
    weak: Weak<MainWindow>,
    ready: Arc<AtomicBool>,
    storage: Option<PathBuf>,
) {
    let client = match reqwest::Client::builder().timeout(Duration::from_secs(7)).build() {
        Ok(client) => client,
        Err(_) => { update(&weak,"Error","Cannot start network client",false); return; }
    };
    let mut session = load_session(storage.as_deref());
    let mut delay = 1u64;
    'outer: loop {
        ready.store(false, Ordering::SeqCst);
        if session.is_none() {
            update(&weak, "Not paired", "Enter the Pi's one-time pairing code in Setup", false);
            let Some(cmd) = rx.recv().await else { break; };
            match cmd {
                Command::Pair { host, code } => {
                    update(&weak,"Pairing","Contacting Pi…",false);
                    match pair(&client,host,code).await {
                        Ok(created) => {
                            if let Err(e) = save_session(storage.as_deref(), &created) {
                                update(&weak,"Pairing not saved",&e,false);
                                // Don't retain pairing if we cannot reconnect safely after restart.
                                continue;
                            }
                            session = Some(created);
                            delay=1;
                        }
                        Err(error) => update(&weak,"Pairing failed",&error,false),
                    }
                }
                Command::Forget => forget_session(storage.as_deref()),
                _ => {}
            }
            continue;
        }

        let data = session.as_ref().expect("session exists").clone();
        update(&weak,"Connecting", &format!("Checking {}",data.base_url),false);
        let result = verify(&client,&data).await;
        match result {
            Ok(false) => {
                forget_session(storage.as_deref());
                session = None;
                update(&weak,"Not paired","Pi revoked this device — pair again",false);
                continue;
            }
            Err(_) => {
                update(&weak,"Disconnected","Pi unavailable; reconnecting",false);
                // Handle configuration changes even while offline.
                tokio::select! {
                    cmd = rx.recv() => {
                        if matches!(cmd, Some(Command::Forget)) {
                            session=None; forget_session(storage.as_deref());
                        } else if let Some(Command::Pair {host,code})=cmd {
                            match pair(&client,host,code).await {
                                Ok(new) => { if save_session(storage.as_deref(),&new).is_ok() {session=Some(new);delay=1;} },
                                Err(e) => update(&weak,"Pairing failed",&e,false)
                            }
                        } else if cmd.is_none() {break;}
                    }
                    _ = time::sleep(Duration::from_secs(delay)) => {}
                }
                delay=(delay*2).min(15);
                continue;
            }
            Ok(true) => {}
        }

        let url = websocket_address(&data.base_url);
        let mut request = match url.into_client_request() {
            Ok(req) => req,
            Err(_) => { update(&weak,"Error","Invalid WebSocket URL",false); break; }
        };
        match HeaderValue::from_str(&data.base_url) {
            Ok(origin) => {request.headers_mut().insert("Origin", origin);},
            Err(_) => {update(&weak,"Error","Invalid Pi origin",false); break;}
        };
        let connected = time::timeout(Duration::from_secs(9),connect_async(request)).await;
        let (mut socket, _) = match connected {
            Ok(Ok(pair)) => pair,
            _ => {
                update(&weak,"Disconnected","WebSocket connection failed",false);
                time::sleep(Duration::from_secs(delay)).await;
                delay=(delay*2).min(15);
                continue;
            }
        };
        if socket.send(Message::Text(json!({"type":"auth","token":data.token}).to_string().into())).await.is_err() {
            continue;
        }
        let handshake = time::timeout(Duration::from_secs(8),socket.next()).await;
        let authorized = match handshake {
            Ok(Some(Ok(Message::Text(message)))) =>
                serde_json::from_str::<Value>(&message).ok().and_then(|x|x["type"].as_str().map(str::to_owned)),
            _ => None,
        };
        if authorized.as_deref() == Some("denied") {
            forget_session(storage.as_deref()); session = None;
            update(&weak,"Not paired","Pi denied this device — pair again",false);
            continue;
        }
        if authorized.as_deref() != Some("ready") {
            update(&weak,"Disconnected","Authentication timed out",false);
            time::sleep(Duration::from_secs(delay)).await;
            delay=(delay*2).min(15);
            continue;
        }

        delay=1;
        while let Ok(cmd) = rx.try_recv() {
            match cmd {
                Command::Forget => { session=None; forget_session(storage.as_deref()); continue 'outer; }
                Command::Pair{host,code} => {
                    match pair(&client,host,code).await {
                        Ok(new) => { if save_session(storage.as_deref(), &new).is_ok() {session=Some(new);continue 'outer;} },
                        Err(e) => update(&weak,"Pairing failed",&e,false),
                    }
                }
                _ => {} // Drop stale input and reconnect commands; never replay.
            }
        }
        ready.store(true, Ordering::SeqCst);
        update(&weak,"Connected",&format!("Connected to {}",data.base_url),true);
        let mut ping = time::interval(Duration::from_secs(12));
        let mut last_pong = Instant::now();
        let mut is_reconfiguring = false;
        loop {
            tokio::select! {
                command = rx.recv() => {
                    match command {
                        None => { ready.store(false,Ordering::SeqCst); return; },
                        Some(Command::Retry) => {is_reconfiguring=true;break;},
                        Some(Command::Forget) => {
                            forget_session(storage.as_deref()); session=None; is_reconfiguring=true; break;
                        }
                        Some(Command::Pair {host,code}) => {
                            match pair(&client,host,code).await {
                                Ok(new) => {
                                    if save_session(storage.as_deref(),&new).is_ok() {
                                        session=Some(new);is_reconfiguring=true;break;
                                    } else {update(&weak,"Error","Could not save pairing",true);}
                                }
                                Err(e) => update(&weak,"Pairing failed",&e,true),
                            }
                        }
                        Some(Command::Input(payload)) => {
                            if socket.send(Message::Text(payload.to_string().into())).await.is_err() {break;}
                        }
                    }
                }
                next = socket.next() => {
                    match next {
                        Some(Ok(Message::Text(raw))) => {
                            if let Ok(v)=serde_json::from_str::<Value>(&raw) {
                                match v["type"].as_str().unwrap_or("") {
                                    "pong" => {last_pong=Instant::now();}
                                    "denied" => {
                                        forget_session(storage.as_deref()); session=None;
                                        update(&weak,"Not paired","Device revoked — pair again",false);
                                        is_reconfiguring=true;break;
                                    }
                                    "app-status" | "clipboard-status" => {
                                        let msg=v["message"].as_str().unwrap_or("Command processed");
                                        let _ = weak.upgrade_in_event_loop({
                                            let m=msg.to_owned();
                                            move |ui| ui.set_status_detail(m.into())
                                        });
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Some(Ok(Message::Ping(bytes))) => {
                            if socket.send(Message::Pong(bytes)).await.is_err(){break;}
                        }
                        Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
                        _ => {}
                    }
                }
                _=ping.tick() => {
                    if last_pong.elapsed()>Duration::from_secs(40){break;}
                    if socket.send(Message::Text(json!({"type":"ping"}).to_string().into())).await.is_err(){break;}
                }
            }
        }
        ready.store(false,Ordering::SeqCst);
        let _ = socket.close(None).await;
        update(&weak,"Disconnected","Connection interrupted; retrying",false);
        if is_reconfiguring {delay=1;continue;}
        // No queued input may be replayed after a disconnect.
        while let Ok(cmd)=rx.try_recv() {
            match cmd {
                Command::Forget => {forget_session(storage.as_deref());session=None;},
                Command::Pair{host,code} => {
                    if let Ok(new)=pair(&client,host,code).await {
                        if save_session(storage.as_deref(),&new).is_ok(){session=Some(new);}
                    }
                },
                _ => {}
            }
        }
        time::sleep(Duration::from_secs(delay)).await;
        delay=(delay*2).min(15);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accepts_pi_local_addresses() {
        assert_eq!(host_base("steamlink.local:3100").unwrap(), "http://steamlink.local:3100");
        assert_eq!(host_base("192.168.1.8:3100").unwrap(), "http://192.168.1.8:3100");
        assert!(host_base("https://example.com").is_err());
        assert!(host_base("127.0.0.1:3100/fake").is_err());
        assert!(host_base("https://user:password@steamlink.local").is_err());
    }
    #[test]
    fn websocket_origin_matches_server() {
        assert_eq!(websocket_address("http://steamlink.local:3100"), "ws://steamlink.local:3100/ws");
        assert_eq!(websocket_address("https://steamlink.local"), "wss://steamlink.local/ws");
    }
}
