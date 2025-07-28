use leptos::prelude::*;
use crate::types::{
    FileItem, CreateFolderArgs, DeleteItemArgs, RenameItemArgs, 
    CopyItemArgs, MoveItemArgs, SearchFilesArgs, PreviewFileArgs, FilePreview
};
use crate::utils::tauri::{invoke, is_tauri_available};

pub async fn create_new_folder(parent_path: String, folder_name: String) {
    if !is_tauri_available() {
        return;
    }

    let args = match serde_wasm_bindgen::to_value(&CreateFolderArgs {
        path: parent_path,
        name: folder_name,
    }) {
        Ok(args) => args,
        Err(_) => return, // Silent fail for now, could add error handling
    };

    let _ = invoke("create_folder", args).await;
}

pub async fn delete_selected_item(item_path: String) {
    if !is_tauri_available() {
        return;
    }

    let args = match serde_wasm_bindgen::to_value(&DeleteItemArgs {
        path: item_path,
    }) {
        Ok(args) => args,
        Err(_) => return, // Silent fail for now, could add error handling
    };

    let _ = invoke("delete_item", args).await;
}

pub async fn rename_selected_item(old_path: String, new_name: String) {
    if !is_tauri_available() {
        return;
    }

    let args = match serde_wasm_bindgen::to_value(&RenameItemArgs {
        old_path,
        new_name,
    }) {
        Ok(args) => args,
        Err(_) => return, // Silent fail for now, could add error handling
    };

    let _ = invoke("rename_item", args).await;
}

pub async fn copy_selected_item(source_path: String, dest_dir: String) -> Result<String, String> {
    if !is_tauri_available() {
        return Err("Tauri not available".to_string());
    }

    let args = match serde_wasm_bindgen::to_value(&CopyItemArgs {
        source: source_path,
        destination: dest_dir,
    }) {
        Ok(args) => args,
        Err(e) => return Err(format!("Failed to serialize arguments: {:?}", e)),
    };

    match invoke("copy_item", args).await {
        Ok(result) => {
            match serde_wasm_bindgen::from_value::<String>(result) {
                Ok(new_path) => Ok(new_path),
                Err(e) => Err(format!("Failed to parse response: {:?}", e)),
            }
        }
        Err(e) => Err(format!("Failed to copy item: {:?}", e)),
    }
}

pub async fn move_selected_item(source_path: String, dest_dir: String) -> Result<String, String> {
    if !is_tauri_available() {
        return Err("Tauri not available".to_string());
    }

    let args = match serde_wasm_bindgen::to_value(&MoveItemArgs {
        source: source_path,
        destination: dest_dir,
    }) {
        Ok(args) => args,
        Err(e) => return Err(format!("Failed to serialize arguments: {:?}", e)),
    };

    match invoke("move_item", args).await {
        Ok(result) => {
            match serde_wasm_bindgen::from_value::<String>(result) {
                Ok(new_path) => Ok(new_path),
                Err(e) => Err(format!("Failed to parse response: {:?}", e)),
            }
        }
        Err(e) => Err(format!("Failed to move item: {:?}", e)),
    }
}

pub async fn search_files(
    directory: String,
    query: String,
    set_search_results: WriteSignal<Option<Vec<FileItem>>>,
    set_searching: WriteSignal<bool>,
    set_error_msg: WriteSignal<Option<String>>,
) {
    set_searching.set(true);
    set_error_msg.set(None);
    set_search_results.set(None);

    if !is_tauri_available() {
        // Return mock search results for browser environment
        let mock_results = vec![
            FileItem {
                name: format!("search_result_{}.txt", query),
                path: format!("/Users/demo/search_result_{}.txt", query),
                is_dir: false,
                size: Some(1024),
                modified: Some("2024-01-15 10:30:00".to_string()),
                icon: "document-text".to_string(),
            },
            FileItem {
                name: format!("{}_folder", query),
                path: format!("/Users/demo/{}_folder", query),
                is_dir: true,
                size: None,
                modified: Some("2024-01-14 15:45:00".to_string()),
                icon: "folder".to_string(),
            },
        ];
        
        set_search_results.set(Some(mock_results));
        set_searching.set(false);
        return;
    }

    let args = match serde_wasm_bindgen::to_value(&SearchFilesArgs {
        directory,
        query,
    }) {
        Ok(args) => args,
        Err(e) => {
            set_error_msg.set(Some(format!("Failed to serialize arguments: {:?}", e)));
            set_searching.set(false);
            return;
        }
    };

    match invoke("search_files", args).await {
        Ok(result) => {
            match serde_wasm_bindgen::from_value::<Vec<FileItem>>(result) {
                Ok(results) => {
                    set_search_results.set(Some(results));
                }
                Err(e) => {
                    set_error_msg.set(Some(format!("Failed to parse search results: {:?}", e)));
                }
            }
        }
        Err(e) => {
            set_error_msg.set(Some(format!("Search failed: {:?}", e)));
        }
    }

    set_searching.set(false);
}

pub async fn preview_file(
    file_path: String,
    set_preview: WriteSignal<Option<FilePreview>>,
    set_loading: WriteSignal<bool>,
    set_error_msg: WriteSignal<Option<String>>,
) {
    set_loading.set(true);
    set_error_msg.set(None);
    set_preview.set(None);

    if !is_tauri_available() {
        // Return mock preview for browser environment
        let mock_preview = FilePreview {
            file_type: "text".to_string(),
            content: format!("Mock preview content for file: {}\n\nThis is a sample text file preview.\nIn the actual Tauri app, this would show the real file content.", file_path),
            size: 1024,
            encoding: "text".to_string(),
        };
        
        set_preview.set(Some(mock_preview));
        set_loading.set(false);
        return;
    }

    let args = match serde_wasm_bindgen::to_value(&PreviewFileArgs {
        path: file_path,
    }) {
        Ok(args) => args,
        Err(e) => {
            set_error_msg.set(Some(format!("Failed to serialize arguments: {:?}", e)));
            set_loading.set(false);
            return;
        }
    };

    match invoke("preview_file", args).await {
        Ok(result) => {
            match serde_wasm_bindgen::from_value::<FilePreview>(result) {
                Ok(preview) => {
                    set_preview.set(Some(preview));
                }
                Err(e) => {
                    set_error_msg.set(Some(format!("Failed to parse preview: {:?}", e)));
                }
            }
        }
        Err(e) => {
            set_error_msg.set(Some(format!("Preview failed: {:?}", e)));
        }
    }

    set_loading.set(false);
}