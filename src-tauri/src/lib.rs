use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use base64::{Engine as _, engine::general_purpose};

#[derive(Debug, Serialize, Deserialize)]
pub struct FileItem {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub modified: Option<String>,
    pub icon: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DirectoryContents {
    pub current_path: String,
    pub parent_path: Option<String>,
    pub items: Vec<FileItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilePreview {
    pub file_type: String,
    pub content: String,
    pub size: u64,
    pub encoding: String, // "text" or "base64"
}

// Security: Validate and sanitize file paths to prevent directory traversal attacks
fn validate_path(path: &str) -> Result<PathBuf, String> {
    let path = Path::new(path);
    
    // Resolve the canonical path to prevent directory traversal
    let canonical = path.canonicalize()
        .map_err(|_| "Invalid or inaccessible path".to_string())?;
    
    // Get home directory for validation
    let home_dir = dirs::home_dir()
        .ok_or("Cannot determine home directory".to_string())?;
    
    // Allow access to home directory and its subdirectories
    if canonical.starts_with(&home_dir) {
        return Ok(canonical);
    }
    
    // Allow access to common system directories (read-only)
    let allowed_system_paths = [
        "/Applications",
        "/System/Applications",
        "/usr/local",
        "/opt",
    ];
    
    for allowed_path in &allowed_system_paths {
        if canonical.starts_with(allowed_path) {
            return Ok(canonical);
        }
    }
    
    Err("Access denied: Path is outside allowed directories".to_string())
}

// Validate path for write operations (more restrictive)
fn validate_write_path(path: &str) -> Result<PathBuf, String> {
    let canonical = validate_path(path)?;
    
    // Only allow write operations in home directory
    let home_dir = dirs::home_dir()
        .ok_or("Cannot determine home directory".to_string())?;
    
    if !canonical.starts_with(&home_dir) {
        return Err("Write access denied: Only home directory is writable".to_string());
    }
    
    Ok(canonical)
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn read_directory(path: String) -> Result<DirectoryContents, String> {
    // Validate path for security
    let dir_path = validate_path(&path)?;
    
    if !dir_path.is_dir() {
        return Err("Path is not a directory".to_string());
    }
    
    let mut items = Vec::new();
    
    match fs::read_dir(&dir_path) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        let metadata = entry.metadata().ok();
                        
                        let name = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Unknown")
                            .to_string();
                        
                        let is_dir = path.is_dir();
                        let size = metadata.as_ref().and_then(|m| if !is_dir { Some(m.len()) } else { None });
                        
                        let modified = metadata.as_ref()
                            .and_then(|m| m.modified().ok())
                            .and_then(|time| {
                                let datetime: DateTime<Utc> = time.into();
                                Some(datetime.format("%Y-%m-%d %H:%M:%S").to_string())
                            });
                        
                        let icon = if is_dir {
                            "folder".to_string()
                        } else {
                            get_file_icon(&name)
                        };
                        
                        items.push(FileItem {
                            name,
                            path: path.to_string_lossy().to_string(),
                            is_dir,
                            size,
                            modified,
                            icon,
                        });
                    }
                    Err(_) => continue,
                }
            }
        }
        Err(e) => return Err(format!("Failed to read directory: {}", e)),
    }
    
    // Sort items: directories first, then files, both alphabetically
    items.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });
    
    let parent_path = dir_path.parent()
        .map(|p| p.to_string_lossy().to_string());
    
    Ok(DirectoryContents {
        current_path: dir_path.to_string_lossy().to_string(),
        parent_path,
        items,
    })
}

#[tauri::command]
async fn get_home_directory() -> Result<String, String> {
    match dirs::home_dir() {
        Some(path) => Ok(path.to_string_lossy().to_string()),
        None => Err("Could not determine home directory".to_string()),
    }
}

#[tauri::command]
async fn create_folder(path: String, name: String) -> Result<String, String> {
    // Validate parent path for write access
    let parent_path = validate_write_path(&path)?;
    
    // Validate folder name to prevent injection
    if name.is_empty() || name.contains('/') || name.contains('\\') || name == "." || name == ".." {
        return Err("Invalid folder name".to_string());
    }
    
    let new_folder_path = parent_path.join(&name);
    
    if new_folder_path.exists() {
        return Err("Folder already exists".to_string());
    }
    
    match fs::create_dir(&new_folder_path) {
        Ok(_) => Ok(new_folder_path.to_string_lossy().to_string()),
        Err(e) => Err(format!("Failed to create folder: {}", e)),
    }
}

#[tauri::command]
async fn delete_item(path: String) -> Result<(), String> {
    // Validate path for write access
    let item_path = validate_write_path(&path)?;
    
    if item_path.is_dir() {
        match fs::remove_dir_all(&item_path) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to delete folder: {}", e)),
        }
    } else {
        match fs::remove_file(&item_path) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to delete file: {}", e)),
        }
    }
}

#[tauri::command]
async fn rename_item(old_path: String, new_name: String) -> Result<String, String> {
    // Validate old path for write access
    let old_item_path = validate_write_path(&old_path)?;
    
    // Validate new name to prevent injection
    if new_name.is_empty() || new_name.contains('/') || new_name.contains('\\') || new_name == "." || new_name == ".." {
        return Err("Invalid file name".to_string());
    }
    
    let parent = old_item_path.parent()
        .ok_or("Cannot determine parent directory")?;
    let new_item_path = parent.join(&new_name);
    
    if new_item_path.exists() {
        return Err("An item with this name already exists".to_string());
    }
    
    match fs::rename(&old_item_path, &new_item_path) {
        Ok(_) => Ok(new_item_path.to_string_lossy().to_string()),
        Err(e) => Err(format!("Failed to rename item: {}", e)),
    }
}

#[tauri::command]
async fn copy_item(source_path: String, dest_dir: String) -> Result<String, String> {
    let source = Path::new(&source_path);
    let dest_parent = Path::new(&dest_dir);
    
    if !source.exists() {
        return Err("Source item does not exist".to_string());
    }
    
    if !dest_parent.exists() || !dest_parent.is_dir() {
        return Err("Destination directory does not exist".to_string());
    }
    
    let file_name = source.file_name()
        .ok_or("Cannot determine file name")?;
    let dest_path = dest_parent.join(file_name);
    
    if dest_path.exists() {
        return Err("An item with this name already exists in destination".to_string());
    }
    
    if source.is_dir() {
        copy_dir_recursive(source, &dest_path)?;
    } else {
        fs::copy(source, &dest_path)
            .map_err(|e| format!("Failed to copy file: {}", e))?;
    }
    
    Ok(dest_path.to_string_lossy().to_string())
}

#[tauri::command]
async fn move_item(source_path: String, dest_dir: String) -> Result<String, String> {
    let source = Path::new(&source_path);
    let dest_parent = Path::new(&dest_dir);
    
    if !source.exists() {
        return Err("Source item does not exist".to_string());
    }
    
    if !dest_parent.exists() || !dest_parent.is_dir() {
        return Err("Destination directory does not exist".to_string());
    }
    
    let file_name = source.file_name()
        .ok_or("Cannot determine file name")?;
    let dest_path = dest_parent.join(file_name);
    
    if dest_path.exists() {
        return Err("An item with this name already exists in destination".to_string());
    }
    
    match fs::rename(source, &dest_path) {
        Ok(_) => Ok(dest_path.to_string_lossy().to_string()),
        Err(e) => Err(format!("Failed to move item: {}", e)),
    }
}

#[tauri::command]
async fn get_item_info(path: String) -> Result<FileItem, String> {
    let item_path = Path::new(&path);
    
    if !item_path.exists() {
        return Err("Item does not exist".to_string());
    }
    
    let metadata = item_path.metadata()
        .map_err(|e| format!("Failed to get metadata: {}", e))?;
    
    let name = item_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown")
        .to_string();
    
    let is_dir = item_path.is_dir();
    let size = if !is_dir { Some(metadata.len()) } else { None };
    
    let modified = metadata.modified().ok()
        .and_then(|time| {
            let datetime: DateTime<Utc> = time.into();
            Some(datetime.format("%Y-%m-%d %H:%M:%S").to_string())
        });
    
    let icon = if is_dir {
        "folder".to_string()
    } else {
        get_file_icon(&name)
    };
    
    Ok(FileItem {
        name,
        path: path.clone(),
        is_dir,
        size,
        modified,
        icon,
    })
}

#[tauri::command]
async fn search_files(directory: String, query: String) -> Result<Vec<FileItem>, String> {
    // Validate directory path for security
    let dir_path = validate_path(&directory)?;
    
    if !dir_path.is_dir() {
        return Err("Path is not a directory".to_string());
    }
    
    if query.trim().is_empty() {
        return Err("Search query cannot be empty".to_string());
    }
    
    let query_lower = query.to_lowercase();
    let mut results = Vec::new();
    
    // Search recursively in the directory
    search_directory_recursive(&dir_path, &query_lower, &mut results)?;
    
    // Sort results: directories first, then files, both alphabetically
    results.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });
    
    Ok(results)
}

#[tauri::command]
async fn preview_file(path: String) -> Result<FilePreview, String> {
    // Validate path for security
    let file_path = validate_path(&path)?;
    
    if !file_path.is_file() {
        return Err("Path is not a file".to_string());
    }
    
    let metadata = file_path.metadata()
        .map_err(|e| format!("Failed to get file metadata: {}", e))?;
    
    let size = metadata.len();
    
    // Limit file size for preview (10MB max)
    if size > 10 * 1024 * 1024 {
        return Err("File too large for preview (max 10MB)".to_string());
    }
    
    let filename = file_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown");
    
    let extension = filename.split('.').last().unwrap_or("").to_lowercase();
    let file_type = get_file_type(&extension);
    
    match file_type.as_str() {
        "text" => {
            // Read as text file
            match fs::read_to_string(&file_path) {
                Ok(content) => Ok(FilePreview {
                    file_type,
                    content,
                    size,
                    encoding: "text".to_string(),
                }),
                Err(_) => {
                    // If UTF-8 reading fails, try reading as binary and show hex preview
                    let bytes = fs::read(&file_path)
                        .map_err(|e| format!("Failed to read file: {}", e))?;
                    
                    let hex_content = bytes.iter()
                        .take(1024) // Show first 1KB as hex
                        .map(|b| format!("{:02x}", b))
                        .collect::<Vec<_>>()
                        .join(" ");
                    
                    Ok(FilePreview {
                        file_type: "binary".to_string(),
                        content: hex_content,
                        size,
                        encoding: "hex".to_string(),
                    })
                }
            }
        }
        "image" => {
            // Read as binary and encode to base64
            let bytes = fs::read(&file_path)
                .map_err(|e| format!("Failed to read image file: {}", e))?;
            
            let base64_content = general_purpose::STANDARD.encode(&bytes);
            
            Ok(FilePreview {
                file_type,
                content: base64_content,
                size,
                encoding: "base64".to_string(),
            })
        }
        _ => {
            Err("File type not supported for preview".to_string())
        }
    }
}

fn search_directory_recursive(dir: &Path, query: &str, results: &mut Vec<FileItem>) -> Result<(), String> {
    match fs::read_dir(dir) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    let metadata = entry.metadata().ok();
                    
                    let name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown")
                        .to_string();
                    
                    // Check if filename contains the search query
                    if name.to_lowercase().contains(query) {
                        let is_dir = path.is_dir();
                        let size = metadata.as_ref().and_then(|m| if !is_dir { Some(m.len()) } else { None });
                        
                        let modified = metadata.as_ref()
                            .and_then(|m| m.modified().ok())
                            .and_then(|time| {
                                let datetime: DateTime<Utc> = time.into();
                                Some(datetime.format("%Y-%m-%d %H:%M:%S").to_string())
                            });
                        
                        let icon = if is_dir {
                            "folder".to_string()
                        } else {
                            get_file_icon(&name)
                        };
                        
                        results.push(FileItem {
                            name,
                            path: path.to_string_lossy().to_string(),
                            is_dir,
                            size,
                            modified,
                            icon,
                        });
                    }
                    
                    // Recursively search subdirectories
                    if path.is_dir() {
                        // Limit recursion depth to prevent infinite loops and performance issues
                        if results.len() < 1000 { // Limit results to prevent memory issues
                            let _ = search_directory_recursive(&path, query, results);
                        }
                    }
                }
            }
        }
        Err(_) => {
            // Silently ignore directories we can't read (permission issues, etc.)
        }
    }
    
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst)
        .map_err(|e| format!("Failed to create directory: {}", e))?;
    
    for entry in fs::read_dir(src)
        .map_err(|e| format!("Failed to read directory: {}", e))? {
        let entry = entry
            .map_err(|e| format!("Failed to read entry: {}", e))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)
                .map_err(|e| format!("Failed to copy file: {}", e))?;
        }
    }
    
    Ok(())
}

fn get_file_icon(filename: &str) -> String {
    let extension = filename.split('.').last().unwrap_or("").to_lowercase();
    
    match extension.as_str() {
        "txt" | "md" | "rtf" => "document-text".to_string(),
        "pdf" => "document".to_string(),
        "doc" | "docx" => "document".to_string(),
        "xls" | "xlsx" => "table".to_string(),
        "ppt" | "pptx" => "presentation".to_string(),
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" => "photo".to_string(),
        "mp4" | "avi" | "mov" | "wmv" | "flv" => "film".to_string(),
        "mp3" | "wav" | "flac" | "aac" => "musical-note".to_string(),
        "zip" | "rar" | "7z" | "tar" | "gz" => "archive-box".to_string(),
        "exe" | "app" | "dmg" => "cog".to_string(),
        "html" | "css" | "js" | "ts" | "json" => "code-bracket".to_string(),
        "rs" | "py" | "java" | "cpp" | "c" => "code-bracket".to_string(),
        _ => "document".to_string(),
    }
}

fn get_file_type(extension: &str) -> String {
    match extension {
        // Text files
        "txt" | "md" | "rtf" | "log" | "csv" | "xml" | "yaml" | "yml" | "toml" | "ini" | "conf" => "text".to_string(),
        "html" | "css" | "js" | "ts" | "json" | "jsx" | "tsx" => "text".to_string(),
        "rs" | "py" | "java" | "cpp" | "c" | "h" | "hpp" | "go" | "php" | "rb" | "swift" => "text".to_string(),
        "sh" | "bash" | "zsh" | "fish" | "ps1" | "bat" | "cmd" => "text".to_string(),
        
        // Image files
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg" | "ico" => "image".to_string(),
        "tiff" | "tif" | "raw" | "cr2" | "nef" | "arw" => "image".to_string(),
        
        // Other types not supported for preview
        _ => "unsupported".to_string(),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            read_directory,
            get_home_directory,
            create_folder,
            delete_item,
            rename_item,
            copy_item,
            move_item,
            get_item_info,
            search_files,
            preview_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
