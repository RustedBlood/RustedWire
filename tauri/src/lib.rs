use kernel::adapters::discovery::sender::HostInfo;
use kernel::application::ports::UserInteractionService;
use kernel::domain::transfer::{FileInfo, SenderInfo};
use serde::Serialize;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

// ─── Serializable wrapper for discovery HostInfo ───

#[derive(Debug, Clone, Serialize)]
pub struct DiscoveredHost {
    pub name: String,
    pub ip: String,
}

impl From<HostInfo> for DiscoveredHost {
    fn from(h: HostInfo) -> Self {
        Self {
            name: h.name,
            ip: h.ip,
        }
    }
}

// ─── Serializable transfer event payload ───

#[derive(Debug, Clone, Serialize)]
pub struct TransferNotification {
    pub sender_name: String,
    pub sender_ip: String,
    pub files: Vec<FileInfo>,
    pub status: String,
}

// ─── App state ───

pub struct AppState {
    server_handle: Mutex<Option<JoinHandle<()>>>,
    broadcast_handle: Mutex<Option<JoinHandle<()>>>,
}

// ─── UserInteractionService: auto-accept + emit event ───

struct TauriUserService {
    app: AppHandle,
}

impl UserInteractionService for TauriUserService {
    fn ask_accept_files(&self, sender_info: &SenderInfo) -> bool {
        let notification = TransferNotification {
            sender_name: sender_info.name.clone(),
            sender_ip: sender_info.ip.clone(),
            files: sender_info.files.clone(),
            status: "received".to_string(),
        };
        let _ = self.app.emit("transfer-request", notification);
        true // auto-accept on local network
    }
}

// ─── Helper: get local IP ───

fn get_local_ip() -> String {
    std::net::UdpSocket::bind("0.0.0.0:0")
        .and_then(|s| {
            s.connect("1.1.1.1:80").ok();
            s.local_addr()
        })
        .map(|a| a.ip().to_string())
        .unwrap_or_else(|_| "127.0.0.1".to_string())
}

// ─── Tauri commands ───

#[tauri::command]
async fn start_server(state: tauri::State<'_, AppState>, app: AppHandle) -> Result<String, String> {
    let mut handle = state.server_handle.lock().await;
    if handle.is_some() {
        return Err("Server already running".into());
    }

    let user_service: Arc<dyn UserInteractionService> =
        Arc::new(TauriUserService { app: app.clone() });

    let task = tokio::spawn(async move {
        kernel::adapters::http::server::start_server(user_service).await;
    });

    *handle = Some(task);

    let local_ip = get_local_ip();
    app.emit(
        "server-status",
        ServerStatusEvent {
            running: true,
            address: format!("{}:8080", local_ip),
        },
    )
    .map_err(|e| e.to_string())?;

    Ok(format!("Server started on {}:8080", local_ip))
}

#[tauri::command]
async fn stop_server(state: tauri::State<'_, AppState>, app: AppHandle) -> Result<String, String> {
    let mut handle = state.server_handle.lock().await;
    if let Some(task) = handle.take() {
        task.abort();
        app.emit(
            "server-status",
            ServerStatusEvent {
                running: false,
                address: String::new(),
            },
        )
        .map_err(|e| e.to_string())?;
        Ok("Server stopped".into())
    } else {
        Err("Server not running".into())
    }
}

#[derive(Debug, Clone, Serialize)]
struct ServerStatusEvent {
    running: bool,
    address: String,
}

#[tauri::command]
async fn get_hostname() -> Result<String, String> {
    Ok(gethostname::gethostname().to_string_lossy().to_string())
}

#[tauri::command]
async fn discover_hosts() -> Result<Vec<DiscoveredHost>, String> {
    let hosts = kernel::adapters::discovery::sender::broadcast_get_recievers()
        .await
        .map_err(|e| e.to_string())?;

    Ok(hosts.into_iter().map(DiscoveredHost::from).collect())
}

#[tauri::command]
async fn start_broadcast(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let mut handle = state.broadcast_handle.lock().await;
    if handle.is_some() {
        return Err("Broadcast already running".into());
    }

    let task = tokio::spawn(async move {
        if let Err(e) = kernel::adapters::discovery::reciever::broadcast_send_msg().await {
            eprintln!("Broadcast error: {}", e);
        }
    });

    *handle = Some(task);
    Ok("Broadcasting presence...".into())
}

#[tauri::command]
async fn stop_broadcast(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let mut handle = state.broadcast_handle.lock().await;
    if let Some(task) = handle.take() {
        task.abort();
        Ok("Broadcast stopped".into())
    } else {
        Err("Broadcast not running".into())
    }
}

#[tauri::command]
async fn send_files(
    host: String,
    file_paths: Vec<String>,
    app: AppHandle,
) -> Result<String, String> {
    if file_paths.is_empty() {
        return Err("No files selected".into());
    }

    let hostname = gethostname::gethostname().to_string_lossy().to_string();
    let local_ip = get_local_ip();

    // Build FileInfo list
    let mut files_info: Vec<FileInfo> = Vec::new();
    for path in &file_paths {
        let meta = tokio::fs::metadata(path)
            .await
            .map_err(|e| format!("Cannot read {}: {}", path, e))?;
        let name = std::path::Path::new(path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        files_info.push(FileInfo {
            name,
            size: meta.len() as i64,
            check_sum: Vec::new(), // checksum not implemented yet
        });
    }

    // Step 1: POST /transfer/prepare
    let sender_info = SenderInfo {
        name: hostname,
        ip: local_ip,
        files: files_info,
    };

    let target = if host.starts_with("http") {
        host.clone()
    } else {
        format!("http://{}", host)
    };

    let client = reqwest::Client::new();
    let prepare_url = format!("{}/transfer/prepare", target);

    let resp = client
        .post(&prepare_url)
        .json(&sender_info)
        .send()
        .await
        .map_err(|e| format!("Failed to connect to {}: {}", target, e))?;

    if !resp.status().is_success() {
        return Err(format!("Prepare failed: HTTP {}", resp.status()));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Bad response: {}", e))?;

    let status = body["status"].as_str().unwrap_or("unknown");
    if status != "Accepted" {
        return Err("Transfer was declined by receiver".into());
    }

    let uuid = body["uuid"]
        .as_str()
        .ok_or("No UUID in response")?
        .to_string();
    let token = body["token"]
        .as_str()
        .ok_or("No token in response")?
        .to_string();

    app.emit(
        "transfer-progress",
        TransferProgressEvent {
            file: String::new(),
            step: "prepared".into(),
            total: file_paths.len() as u32,
            current: 0,
        },
    )
    .map_err(|e| e.to_string())?;

    // Step 2: Upload each file
    let upload_url = format!("{}/transfer/upload/{}", target, uuid);

    for (i, path) in file_paths.iter().enumerate() {
        let file_name = std::path::Path::new(path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let file_data = tokio::fs::read(path)
            .await
            .map_err(|e| format!("Cannot read {}: {}", path, e))?;

        let part = reqwest::multipart::Part::bytes(file_data).file_name(file_name.clone());
        let form = reqwest::multipart::Form::new().part("file", part);

        app.emit(
            "transfer-progress",
            TransferProgressEvent {
                file: file_name.clone(),
                step: "uploading".into(),
                total: file_paths.len() as u32,
                current: i as u32 + 1,
            },
        )
        .map_err(|e| e.to_string())?;

        let resp = client
            .post(&upload_url)
            .header("token", &token)
            .multipart(form)
            .send()
            .await
            .map_err(|e| format!("Upload failed for {}: {}", file_name, e))?;

        if !resp.status().is_success() {
            return Err(format!(
                "Upload failed for {}: HTTP {}",
                file_name,
                resp.status()
            ));
        }
    }

    app.emit(
        "transfer-progress",
        TransferProgressEvent {
            file: String::new(),
            step: "done".into(),
            total: file_paths.len() as u32,
            current: file_paths.len() as u32,
        },
    )
    .map_err(|e| e.to_string())?;

    Ok(format!("Successfully sent {} file(s)", file_paths.len()))
}

#[derive(Debug, Clone, Serialize)]
struct TransferProgressEvent {
    file: String,
    step: String,
    total: u32,
    current: u32,
}

#[tauri::command]
async fn pick_files(app: AppHandle) -> Result<Vec<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let result = app
        .dialog()
        .file()
        .add_filter("All Files", &["*"])
        .blocking_pick_files();

    match result {
        Some(paths) => {
            let files: Vec<String> = paths
                .iter()
                .filter_map(|p| p.as_path().map(|pb| pb.to_string_lossy().to_string()))
                .collect();
            Ok(files)
        }
        None => Ok(Vec::new()),
    }
}

// ─── Tauri setup ───

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .manage(AppState {
            server_handle: Mutex::new(None),
            broadcast_handle: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            start_server,
            stop_server,
            get_hostname,
            discover_hosts,
            start_broadcast,
            stop_broadcast,
            send_files,
            pick_files,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
