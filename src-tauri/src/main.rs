#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::{Arc, Mutex};
use std::io::Read;
use std::net::TcpStream;
use std::path::Path;
use ssh2::Session;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FileInfo {
  name: String,
  path: String,
  is_dir: bool,
  size: u64,
  modified: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct DiagnosticsResponse {
  success: bool,
  #[serde(default)]
  files: Vec<FileInfo>,
  #[serde(default)]
  message: String,
}

enum ConnectionType {
  Phoenix,
  SSH,
}

struct PhoenixConnection {
  client: Client,
  host: String,
  port: u16,
}

struct SSHConnection {
  session: Session,
  host: String,
  username: String,
}

struct RioConnection {
  connection_type: Option<ConnectionType>,
  phoenix: Option<PhoenixConnection>,
  ssh: Option<SSHConnection>,
}

#[tauri::command]
async fn connect_phoenix(
  host: String,
  connection: State<'_, Arc<Mutex<RioConnection>>>,
) -> Result<String, String> {
  let client = Client::new();
  let url = format!("http://{}:5800/api/v1/status", host);
  
  match client.get(&url).send().await {
    Ok(response) => {
      if response.status().is_success() {
        let mut conn = connection.lock().unwrap();
        conn.connection_type = Some(ConnectionType::Phoenix);
        conn.phoenix = Some(PhoenixConnection {
          client,
          host: host.clone(),
          port: 5800,
        });
        conn.ssh = None;
        
        Ok(format!("✓ Connected to Phoenix Tuner X on {}", host))
      } else {
        Err("Failed to connect to Phoenix diagnostic server".to_string())
      }
    }
    Err(e) => {
      Err(format!("Connection error: {}. Make sure Phoenix Tuner X is running on port 5800.", e))
    }
  }
}

#[tauri::command]
async fn connect_ssh(
  host: String,
  username: String,
  password: String,
  connection: State<'_, Arc<Mutex<RioConnection>>>,
) -> Result<String, String> {
  let tcp = TcpStream::connect(format!("{}:22", host))
    .map_err(|e| format!("Failed to connect to SSH: {}", e))?;

  let mut session = Session::new().map_err(|e| format!("SSH error: {}", e))?;
  session
    .set_tcp_stream(tcp);
  session
    .handshake()
    .map_err(|e| format!("Handshake failed: {}", e))?;
  session
    .userauth_password(&username, &password)
    .map_err(|e| format!("Authentication failed: {}", e))?;

  let mut conn = connection.lock().unwrap();
  conn.connection_type = Some(ConnectionType::SSH);
  conn.ssh = Some(SSHConnection {
    session,
    host: host.clone(),
    username: username.clone(),
  });
  conn.phoenix = None;

  Ok(format!("✓ Connected via SSH to {} as {}", host, username))
}

#[tauri::command]
async fn list_files(
  path: String,
  connection: State<'_, Arc<Mutex<RioConnection>>>,
) -> Result<Vec<FileInfo>, String> {
  let conn = connection.lock().unwrap();
  
  match &conn.connection_type {
    Some(ConnectionType::Phoenix) => {
      list_files_phoenix(&conn, &path).await
    }
    Some(ConnectionType::SSH) => {
      list_files_ssh(&conn, &path)
    }
    None => Err("Not connected. Please connect to a RoboRIO first.".to_string()),
  }
}

async fn list_files_phoenix(conn: &RioConnection, path: &str) -> Result<Vec<FileInfo>, String> {
  let phoenix = conn.phoenix.as_ref().ok_or("Phoenix connection lost")?;
  let encoded_path = base64::encode(path);
  let url = format!("http://{}:{}/api/v1/files?path={}", phoenix.host, phoenix.port, encoded_path);

  match phoenix.client.get(&url).send().await {
    Ok(response) => {
      let data: DiagnosticsResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

      if data.success {
        Ok(data.files)
      } else {
        Err(data.message)
      }
    }
    Err(e) => Err(format!("Request failed: {}", e)),
  }
}

fn list_files_ssh(conn: &RioConnection, path: &str) -> Result<Vec<FileInfo>, String> {
  let ssh = conn.ssh.as_ref().ok_or("SSH connection lost")?;
  let mut sftp = ssh.session
    .sftp()
    .map_err(|e| format!("SFTP error: {}", e))?;

  let entries = sftp
    .readdir(Path::new(path))
    .map_err(|e| format!("Failed to read directory: {}", e))?;

  let files = entries
    .iter()
    .map(|(path, stat)| FileInfo {
      name: path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string(),
      path: path.to_string_lossy().to_string(),
      is_dir: stat.is_dir(),
      size: stat.size.unwrap_or(0),
      modified: stat.mtime.unwrap_or(0),
    })
    .collect();

  Ok(files)
}

#[tauri::command]
async fn download_file(
  remote_path: String,
  local_path: String,
  connection: State<'_, Arc<Mutex<RioConnection>>>,
) -> Result<String, String> {
  let conn = connection.lock().unwrap();
  
  match &conn.connection_type {
    Some(ConnectionType::Phoenix) => {
      download_file_phoenix(&conn, &remote_path, &local_path).await
    }
    Some(ConnectionType::SSH) => {
      download_file_ssh(&conn, &remote_path, &local_path)
    }
    None => Err("Not connected".to_string()),
  }
}

async fn download_file_phoenix(conn: &RioConnection, remote_path: &str, local_path: &str) -> Result<String, String> {
  let phoenix = conn.phoenix.as_ref().ok_or("Phoenix connection lost")?;
  let encoded_path = base64::encode(remote_path);
  let url = format!("http://{}:{}/api/v1/download?path={}", phoenix.host, phoenix.port, encoded_path);

  match phoenix.client.get(&url).send().await {
    Ok(response) => {
      let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to download: {}", e))?;

      std::fs::write(local_path, bytes)
        .map_err(|e| format!("Failed to write file: {}", e))?;

      Ok(format!("Downloaded to {}", local_path))
    }
    Err(e) => Err(format!("Download failed: {}", e)),
  }
}

fn download_file_ssh(conn: &RioConnection, remote_path: &str, local_path: &str) -> Result<String, String> {
  let ssh = conn.ssh.as_ref().ok_or("SSH connection lost")?;
  let mut sftp = ssh.session
    .sftp()
    .map_err(|e| format!("SFTP error: {}", e))?;

  let mut remote_file = sftp
    .open(Path::new(remote_path))
    .map_err(|e| format!("Failed to open file: {}", e))?;

  let mut contents = Vec::new();
  remote_file
    .read_to_end(&mut contents)
    .map_err(|e| format!("Failed to read file: {}", e))?;

  std::fs::write(local_path, contents)
    .map_err(|e| format!("Failed to write file: {}", e))?;

  Ok(format!("Downloaded to {}", local_path))
}

#[tauri::command]
async fn upload_file(
  local_path: String,
  remote_path: String,
  connection: State<'_, Arc<Mutex<RioConnection>>>,
) -> Result<String, String> {
  let conn = connection.lock().unwrap();
  
  match &conn.connection_type {
    Some(ConnectionType::Phoenix) => {
      upload_file_phoenix(&conn, &local_path, &remote_path).await
    }
    Some(ConnectionType::SSH) => {
      upload_file_ssh(&conn, &local_path, &remote_path)
    }
    None => Err("Not connected".to_string()),
  }
}

async fn upload_file_phoenix(conn: &RioConnection, local_path: &str, remote_path: &str) -> Result<String, String> {
  let phoenix = conn.phoenix.as_ref().ok_or("Phoenix connection lost")?;
  
  let contents = std::fs::read(local_path)
    .map_err(|e| format!("Failed to read local file: {}", e))?;

  let encoded_path = base64::encode(remote_path);
  let url = format!("http://{}:{}/api/v1/upload?path={}", phoenix.host, phoenix.port, encoded_path);

  match phoenix.client
    .post(&url)
    .body(contents)
    .send()
    .await
  {
    Ok(response) => {
      let data: DiagnosticsResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

      if data.success {
        Ok(format!("Uploaded to {}", remote_path))
      } else {
        Err(data.message)
      }
    }
    Err(e) => Err(format!("Upload failed: {}", e)),
  }
}

fn upload_file_ssh(conn: &RioConnection, local_path: &str, remote_path: &str) -> Result<String, String> {
  let ssh = conn.ssh.as_ref().ok_or("SSH connection lost")?;
  let mut sftp = ssh.session
    .sftp()
    .map_err(|e| format!("SFTP error: {}", e))?;

  let contents = std::fs::read(local_path)
    .map_err(|e| format!("Failed to read local file: {}", e))?;

  let mut remote_file = sftp
    .create(Path::new(remote_path))
    .map_err(|e| format!("Failed to create remote file: {}", e))?;

  std::io::Write::write_all(&mut remote_file, &contents)
    .map_err(|e| format!("Failed to write file: {}", e))?;

  Ok(format!("Uploaded to {}", remote_path))
}

#[tauri::command]
async fn delete_file(
  path: String,
  connection: State<'_, Arc<Mutex<RioConnection>>>,
) -> Result<String, String> {
  let conn = connection.lock().unwrap();
  
  match &conn.connection_type {
    Some(ConnectionType::Phoenix) => {
      delete_file_phoenix(&conn, &path).await
    }
    Some(ConnectionType::SSH) => {
      delete_file_ssh(&conn, &path)
    }
    None => Err("Not connected".to_string()),
  }
}

async fn delete_file_phoenix(conn: &RioConnection, path: &str) -> Result<String, String> {
  let phoenix = conn.phoenix.as_ref().ok_or("Phoenix connection lost")?;
  let encoded_path = base64::encode(path);
  let url = format!("http://{}:{}/api/v1/delete?path={}", phoenix.host, phoenix.port, encoded_path);

  match phoenix.client.delete(&url).send().await {
    Ok(response) => {
      let data: DiagnosticsResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

      if data.success {
        Ok("Deleted successfully".to_string())
      } else {
        Err(data.message)
      }
    }
    Err(e) => Err(format!("Delete failed: {}", e)),
  }
}

fn delete_file_ssh(conn: &RioConnection, path: &str) -> Result<String, String> {
  let ssh = conn.ssh.as_ref().ok_or("SSH connection lost")?;
  let mut sftp = ssh.session
    .sftp()
    .map_err(|e| format!("SFTP error: {}", e))?;

  sftp
    .unlink(Path::new(path))
    .map_err(|e| format!("Failed to delete: {}", e))?;

  Ok("Deleted successfully".to_string())
}

fn main() {
  let connection = Arc::new(Mutex::new(RioConnection {
    connection_type: None,
    phoenix: None,
    ssh: None,
  }));

  tauri::Builder::default()
    .manage(connection)
    .invoke_handler(tauri::generate_handler![
      connect_phoenix,
      connect_ssh,
      list_files,
      download_file,
      upload_file,
      delete_file
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
