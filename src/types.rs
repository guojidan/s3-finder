use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileItem {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub modified: Option<String>,
    pub icon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryContents {
    pub current_path: String,
    pub parent_path: Option<String>,
    pub items: Vec<FileItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadDirArgs {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFolderArgs {
    pub path: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteItemArgs {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameItemArgs {
    pub old_path: String,
    pub new_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyItemArgs {
    pub source: String,
    pub destination: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveItemArgs {
    pub source: String,
    pub destination: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilesArgs {
    pub directory: String,
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewFileArgs {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePreview {
    pub file_type: String,
    pub content: String,
    pub size: u64,
    pub encoding: String, // "text" or "base64"
}
