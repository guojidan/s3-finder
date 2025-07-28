use leptos::task::spawn_local;
use leptos::{
    ev::{KeyboardEvent, MouseEvent},
    prelude::*,
};
use wasm_bindgen::JsValue;

// Import our modules
use crate::components::file_icon::FileIcon;
use crate::services::file_service::*;
use crate::types::*;
use crate::utils::format::format_file_size;
use crate::utils::tauri::{invoke, is_tauri_available};

#[derive(Clone, Debug)]
pub struct ColumnData {
    pub path: String,
    pub contents: DirectoryContents,
}

#[component]
pub fn App() -> impl IntoView {
    let (current_path, set_current_path) = signal(String::new());
    let (columns, set_columns) = signal(Vec::<ColumnData>::new());
    let (loading, set_loading) = signal(false);
    let (error_msg, set_error_msg) = signal(Option::<String>::None);
    let (sidebar_collapsed, set_sidebar_collapsed) = signal(false);
    let (selected_item, set_selected_item) = signal(Option::<String>::None);
    let (selected_column_index, set_selected_column_index) = signal(Option::<usize>::None);
    let (focused_item, set_focused_item) = signal(Option::<String>::None);
    let (focused_column_index, set_focused_column_index) = signal(Option::<usize>::None);
    let (context_menu_visible, set_context_menu_visible) = signal(false);
    let (context_menu_pos, set_context_menu_pos) = signal((0, 0));
    let (show_new_folder_dialog, set_show_new_folder_dialog) = signal(false);
    let (new_folder_name, set_new_folder_name) = signal(String::new());
    let (show_rename_dialog, set_show_rename_dialog) = signal(false);
    let (rename_item_name, set_rename_item_name) = signal(String::new());
    let (rename_item_path, set_rename_item_path) = signal(String::new());

    // Copy/Move states
    let (clipboard_item, set_clipboard_item) = signal(Option::<String>::None);
    let (clipboard_operation, set_clipboard_operation) = signal(Option::<String>::None); // "copy" or "cut"

    // Search states
    let (search_query, set_search_query) = signal(String::new());
    let (search_results, set_search_results) = signal(Option::<Vec<FileItem>>::None);
    let (searching, set_searching) = signal(false);
    let (search_mode, set_search_mode) = signal(false);

    // Preview states
    // let (show_preview, set_show_preview) = signal(false);
    let (preview_content, set_preview_content) = signal(Option::<FilePreview>::None);
    let (preview_loading, set_preview_loading) = signal(false);
    let (preview_error, set_preview_error) = signal(Option::<String>::None);

    // Zoom states
    let (zoom_level, set_zoom_level) = signal(1.0f64);
    let min_zoom = 0.5f64;
    let max_zoom = 2.0f64;
    let zoom_step = 0.1f64;

    // Theme states
    let (theme, set_theme) = signal("auto".to_string()); // "light", "dark", "auto"

    // Load directory and add to columns
    let load_directory_column = move |path: String, column_index: Option<usize>| {
        let path_clone = path.clone();
        set_loading.set(true);
        spawn_local(async move {
            if is_tauri_available() {
                match invoke(
                    "read_directory",
                    serde_wasm_bindgen::to_value(&ReadDirArgs {
                        path: path_clone.clone(),
                    })
                    .unwrap(),
                )
                .await
                {
                    Ok(result) => {
                        match serde_wasm_bindgen::from_value::<DirectoryContents>(result) {
                            Ok(contents) => {
                                let new_col_index = set_columns.update_untracked(|cols| {
                                    if let Some(index) = column_index {
                                        // Replace from this column onwards
                                        cols.truncate(index);
                                        cols.push(ColumnData {
                                            path: path_clone.clone(),
                                            contents,
                                        });
                                        index
                                    } else {
                                        // Add new column
                                        cols.push(ColumnData {
                                            path: path_clone.clone(),
                                            contents,
                                        });
                                        cols.len() - 1
                                    }
                                });
                                set_current_path.set(path_clone);
                                set_error_msg.set(None);
                                // Set focus to the new/updated column
                                set_selected_column_index.set(Some(new_col_index));
                                set_selected_item.set(None);

                                // Auto-scroll to the rightmost column
                                if let Some(window) = web_sys::window() {
                                    if let Some(document) = window.document() {
                                        if let Some(container) = document
                                            .query_selector(".columns-container")
                                            .ok()
                                            .flatten()
                                        {
                                            let container: web_sys::Element = container;
                                            container.set_scroll_left(container.scroll_width());
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                set_error_msg.set(Some(format!(
                                    "Failed to parse directory contents: {e}"
                                )));
                            }
                        }
                    }
                    Err(e) => {
                        set_error_msg.set(Some(format!("Failed to load directory: {e:?}")));
                    }
                }
            } else {
                // Browser environment - use mock data
                let mock_contents = DirectoryContents {
                    current_path: path_clone.clone(),
                    parent_path: if path_clone == "/" {
                        None
                    } else {
                        Some("/".to_string())
                    },
                    items: vec![
                        FileItem {
                            name: "Documents".to_string(),
                            path: format!("{path_clone}/Documents"),
                            is_dir: true,
                            size: None,
                            modified: Some("2024-01-15".to_string()),
                            icon: "folder".to_string(),
                        },
                        FileItem {
                            name: "example.txt".to_string(),
                            path: format!("{path_clone}/example.txt"),
                            is_dir: false,
                            size: Some(1024),
                            modified: Some("2024-01-15".to_string()),
                            icon: "text".to_string(),
                        },
                    ],
                };

                let new_col_index = set_columns.update_untracked(|cols| {
                    if let Some(index) = column_index {
                        cols.truncate(index);
                        cols.push(ColumnData {
                            path: path_clone.clone(),
                            contents: mock_contents,
                        });
                        index
                    } else {
                        cols.push(ColumnData {
                            path: path_clone.clone(),
                            contents: mock_contents,
                        });
                        cols.len() - 1
                    }
                });
                set_current_path.set(path_clone);
                set_error_msg.set(None);
                // Set focus to the new/updated column
                set_selected_column_index.set(Some(new_col_index));
                set_selected_item.set(None);

                // Auto-scroll to the rightmost column
                if let Some(window) = web_sys::window() {
                    if let Some(document) = window.document() {
                        if let Some(container) =
                            document.query_selector(".columns-container").ok().flatten()
                        {
                            let container: web_sys::Element = container;
                            container.set_scroll_left(container.scroll_width());
                        }
                    }
                }
            }
            set_loading.set(false);
        });
    };

    // Initialize with home directory
    Effect::new(move |_| {
        spawn_local(async move {
            if is_tauri_available() {
                match invoke("get_home_directory", JsValue::NULL).await {
                    Ok(home_path_value) => {
                        if let Some(home_path) = home_path_value.as_string() {
                            load_directory_column(home_path, None);
                        } else {
                            load_directory_column("/Users/demo".to_string(), None);
                        }
                    }
                    Err(_) => {
                        load_directory_column("/Users/demo".to_string(), None);
                    }
                }
            } else {
                load_directory_column("/Users/demo".to_string(), None);
            }
        });
    });

    // Theme management effect
    Effect::new(move |_| {
        let current_theme = theme.get();
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(body) = document.body() {
                    // Set the data-theme attribute on the body
                    let _ = body.set_attribute("data-theme", &current_theme);
                }
            }
        }
    });

    // Auto-preview effect when item is selected
    Effect::new(move |_| {
        if let Some(selected_path) = selected_item.get() {
            // Check if the selected item is a file (not a directory)
            let is_file = if let Some(col_index) = selected_column_index.get() {
                if let Some(column) = columns.get().get(col_index) {
                    column
                        .contents
                        .items
                        .iter()
                        .find(|item| item.path == selected_path)
                        .map(|item| !item.is_dir)
                        .unwrap_or(false)
                } else {
                    false
                }
            } else if let Some(search_results) = search_results.get() {
                search_results
                    .iter()
                    .find(|item| item.path == selected_path)
                    .map(|item| !item.is_dir)
                    .unwrap_or(false)
            } else {
                false
            };

            if is_file {
                // Only preview files, not directories
                spawn_local(async move {
                    preview_file(
                        selected_path,
                        set_preview_content,
                        set_preview_loading,
                        set_preview_error,
                    )
                    .await;
                });
            } else {
                // Clear preview for directories - don't show preview for directories
                set_preview_content.set(None);
                set_preview_error.set(None);
                set_preview_loading.set(false);
            }
        } else {
            // Clear preview when no item is selected
            set_preview_content.set(None);
            set_preview_error.set(None);
            set_preview_loading.set(false);
        }
    });

    let navigate_to = move |path: String| {
        set_columns.set(Vec::new());
        load_directory_column(path, None);
    };

    let go_up = move |_: MouseEvent| {
        let cols = columns.get();
        if cols.len() > 1 {
            // Remove the last column
            set_columns.update(|cols| {
                cols.pop();
            });
            if let Some(last_col) = cols.get(cols.len() - 2) {
                set_current_path.set(last_col.path.clone());
            }
        } else if let Some(first_col) = cols.first() {
            if let Some(parent) = &first_col.contents.parent_path {
                navigate_to(parent.clone());
            }
        }
    };

    // Refresh current column
    let refresh_current_column = move || {
        if let Some(last_col) = columns.get().last() {
            let path = last_col.path.clone();
            let col_index = columns.get().len() - 1;
            load_directory_column(path, Some(col_index));
        }
    };

    // Navigation helper functions
    // Scroll to focused item
    let scroll_to_focused_item = move || {
        if focused_item.get().is_some() {
            if let Some(col_index) = focused_column_index.get() {
                spawn_local(async move {
                    // Wait a bit for DOM to update
                    wasm_bindgen_futures::JsFuture::from(js_sys::Promise::new(
                        &mut |resolve, _| {
                            web_sys::window()
                                .unwrap()
                                .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 50)
                                .unwrap();
                        },
                    ))
                    .await
                    .unwrap();

                    if let Some(window) = web_sys::window() {
                        if let Some(document) = window.document() {
                            // Find the focused item element
                            let selector =
                                format!(".column:nth-child({}) .file-item.focused", col_index + 1);
                            if let Some(focused_element) =
                                document.query_selector(&selector).ok().flatten()
                            {
                                // Scroll the focused item into view with smooth behavior
                                focused_element.scroll_into_view_with_bool(true);
                            }
                        }
                    }
                });
            }
        }
    };

    // Scroll columns container to the rightmost position
    let scroll_to_rightmost_column = move || {
        spawn_local(async move {
            // Wait a bit for DOM to update
            wasm_bindgen_futures::JsFuture::from(js_sys::Promise::new(&mut |resolve, _| {
                web_sys::window()
                    .unwrap()
                    .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 100)
                    .unwrap();
            }))
            .await
            .unwrap();

            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Some(container) =
                        document.query_selector(".columns-container").ok().flatten()
                    {
                        let container: web_sys::Element = container;
                        container.set_scroll_left(container.scroll_width());
                    }
                }
            }
        });
    };

    let navigate_up = move || {
        if let Some(col_index) = focused_column_index.get() {
            if let Some(column) = columns.get().get(col_index) {
                let items = &column.contents.items;
                if let Some(current_focused) = focused_item.get() {
                    if let Some(current_index) =
                        items.iter().position(|item| item.path == current_focused)
                    {
                        if current_index > 0 {
                            let new_focused = &items[current_index - 1];
                            set_focused_item.set(Some(new_focused.path.clone()));
                            scroll_to_focused_item();
                        }
                    }
                } else if !items.is_empty() {
                    // Focus on last item if nothing is focused
                    let last_item = &items[items.len() - 1];
                    set_focused_item.set(Some(last_item.path.clone()));
                    set_focused_column_index.set(Some(col_index));
                    scroll_to_focused_item();
                }
            }
        } else if !columns.get().is_empty() {
            // Focus on the last column if no column is focused
            let last_col_index = columns.get().len() - 1;
            set_focused_column_index.set(Some(last_col_index));
            if let Some(column) = columns.get().get(last_col_index) {
                if !column.contents.items.is_empty() {
                    let last_item = &column.contents.items[column.contents.items.len() - 1];
                    set_focused_item.set(Some(last_item.path.clone()));
                    scroll_to_focused_item();
                }
            }
        }
    };

    let navigate_down = move || {
        if let Some(col_index) = focused_column_index.get() {
            if let Some(column) = columns.get().get(col_index) {
                let items = &column.contents.items;
                if let Some(current_focused) = focused_item.get() {
                    if let Some(current_index) =
                        items.iter().position(|item| item.path == current_focused)
                    {
                        if current_index < items.len() - 1 {
                            let new_focused = &items[current_index + 1];
                            set_focused_item.set(Some(new_focused.path.clone()));
                            scroll_to_focused_item();
                        }
                    }
                } else if !items.is_empty() {
                    // Focus on first item if nothing is focused
                    let first_item = &items[0];
                    set_focused_item.set(Some(first_item.path.clone()));
                    set_focused_column_index.set(Some(col_index));
                    scroll_to_focused_item();
                }
            }
        } else if !columns.get().is_empty() {
            // Focus on the first column if no column is focused
            set_focused_column_index.set(Some(0));
            if let Some(column) = columns.get().first() {
                if !column.contents.items.is_empty() {
                    let first_item = &column.contents.items[0];
                    set_focused_item.set(Some(first_item.path.clone()));
                    scroll_to_focused_item();
                }
            }
        }
    };

    let navigate_left = move || {
        if let Some(col_index) = focused_column_index.get() {
            if col_index > 0 {
                let new_col_index = col_index - 1;
                set_focused_column_index.set(Some(new_col_index));

                // Try to maintain the same item name if possible
                if let Some(current_focused) = focused_item.get() {
                    if let Some(current_name) = std::path::Path::new(&current_focused).file_name() {
                        if let Some(column) = columns.get().get(new_col_index) {
                            if let Some(matching_item) = column.contents.items.iter().find(|item| {
                                std::path::Path::new(&item.path).file_name() == Some(current_name)
                            }) {
                                set_focused_item.set(Some(matching_item.path.clone()));
                                scroll_to_focused_item();
                                return;
                            }
                        }
                    }
                }

                // If no matching item, focus on first item in the column
                if let Some(column) = columns.get().get(new_col_index) {
                    if !column.contents.items.is_empty() {
                        let first_item = &column.contents.items[0];
                        set_focused_item.set(Some(first_item.path.clone()));
                        scroll_to_focused_item();
                    }
                }
            }
        }
    };

    let navigate_right = move || {
        if let Some(col_index) = focused_column_index.get() {
            if col_index < columns.get().len() - 1 {
                let new_col_index = col_index + 1;
                set_focused_column_index.set(Some(new_col_index));

                // Try to maintain the same item name if possible
                if let Some(current_focused) = focused_item.get() {
                    if let Some(current_name) = std::path::Path::new(&current_focused).file_name() {
                        if let Some(column) = columns.get().get(new_col_index) {
                            if let Some(matching_item) = column.contents.items.iter().find(|item| {
                                std::path::Path::new(&item.path).file_name() == Some(current_name)
                            }) {
                                set_focused_item.set(Some(matching_item.path.clone()));
                                scroll_to_focused_item();
                                return;
                            }
                        }
                    }
                }

                // If no matching item, focus on first item in the column
                if let Some(column) = columns.get().get(new_col_index) {
                    if !column.contents.items.is_empty() {
                        let first_item = &column.contents.items[0];
                        set_focused_item.set(Some(first_item.path.clone()));
                        scroll_to_focused_item();
                    }
                }
            }
        }
    };

    let activate_focused_item = move |path: String| {
        // Check if it's a directory
        if let Some(col_index) = focused_column_index.get() {
            if let Some(column) = columns.get().get(col_index) {
                if let Some(item) = column.contents.items.iter().find(|item| item.path == path) {
                    if item.is_dir {
                        // Navigate into directory
                        load_directory_column(path.clone(), Some(col_index + 1));
                        set_selected_item.set(Some(path.clone()));
                        set_selected_column_index.set(Some(col_index));

                        // Immediately set focus to the new column
                        let new_col_index = col_index + 1;
                        set_focused_column_index.set(Some(new_col_index));

                        // Focus on first item in new column after it loads
                        spawn_local(async move {
                            // Wait a bit for the new column to load
                            wasm_bindgen_futures::JsFuture::from(js_sys::Promise::new(
                                &mut |resolve, _| {
                                    web_sys::window()
                                        .unwrap()
                                        .set_timeout_with_callback_and_timeout_and_arguments_0(
                                            &resolve, 100,
                                        )
                                        .unwrap();
                                },
                            ))
                            .await
                            .unwrap();

                            if let Some(new_column) = columns.get().get(new_col_index) {
                                if !new_column.contents.items.is_empty() {
                                    let first_item = &new_column.contents.items[0];
                                    set_focused_item.set(Some(first_item.path.clone()));
                                }
                            }
                        });
                    } else {
                        // Select file
                        set_selected_item.set(Some(path));
                        set_selected_column_index.set(Some(col_index));
                    }
                }
            }
        }
    };

    // Keyboard navigation handlers
    let handle_keyboard_navigation = move |e: KeyboardEvent| {
        let key = e.key();

        match key.as_str() {
            "ArrowUp" => {
                e.prevent_default();
                navigate_up();
            }
            "ArrowDown" => {
                e.prevent_default();
                navigate_down();
            }
            "ArrowLeft" => {
                e.prevent_default();
                navigate_left();
            }
            "ArrowRight" => {
                e.prevent_default();
                navigate_right();
            }
            "Enter" => {
                e.prevent_default();
                if let Some(focused_path) = focused_item.get() {
                    activate_focused_item(focused_path);
                }
            }
            "Escape" => {
                e.prevent_default();
                set_focused_item.set(None);
                set_focused_column_index.set(None);
            }
            _ => {}
        }
    };

    // Theme toggle function
    let toggle_theme = move |_: MouseEvent| {
        let current_theme = theme.get();
        let new_theme = match current_theme.as_str() {
            "light" => "dark",
            _ => "light", // "dark" or any other value goes to "light"
        };
        set_theme.set(new_theme.to_string());
    };

    let toggle_sidebar = move |_: MouseEvent| {
        set_sidebar_collapsed.update(|collapsed| *collapsed = !*collapsed);
    };

    view! {
        <div
            class="finder-app"
            tabindex="0"
            on:keydown=handle_keyboard_navigation
        >
            // Toolbar
            <div class="toolbar">
                <div class="toolbar-left">
                    <button class="toolbar-btn" on:click=toggle_sidebar>
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M3 18h18v-2H3v2zm0-5h18v-2H3v2zm0-7v2h18V6H3z"/>
                        </svg>
                    </button>
                    <button class="toolbar-btn" on:click=go_up>
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"/>
                        </svg>
                    </button>

                    <button
                        class="toolbar-btn"
                        on:click=move |_| set_show_new_folder_dialog.set(true)
                        title="New Folder"
                    >
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M20 6h-2l-2-2H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2zm-1 8h-3v3h-2v-3h-3v-2h3V9h2v3h3v2z"/>
                        </svg>
                    </button>
                </div>
                <div class="toolbar-center">
                    <div class="path-bar">
                        {move || current_path.get()}
                    </div>
                </div>
                <div class="toolbar-right">
                    <div class="search-container">
                        <input
                            type="text"
                            class="search-input"
                            placeholder="Search files..."
                            prop:value=move || search_query.get()
                            on:input=move |e| set_search_query.set(event_target_value(&e))
                            on:keydown=move |e| {
                                if e.key() == "Enter" {
                                    let query = search_query.get();
                                    if !query.trim().is_empty() {
                                        set_search_mode.set(true);
                                        spawn_local(async move {
                                            search_files(
                                                current_path.get(),
                                                query,
                                                set_search_results,
                                                set_searching,
                                                set_error_msg
                                            ).await;
                                        });
                                    }
                                }
                            }
                        />
                        <button
                            class="search-btn"
                            on:click=move |_| {
                                let query = search_query.get();
                                if !query.trim().is_empty() {
                                    set_search_mode.set(true);
                                    spawn_local(async move {
                                        search_files(
                                            current_path.get(),
                                            query,
                                            set_search_results,
                                            set_searching,
                                            set_error_msg
                                        ).await;
                                    });
                                } else {
                                    set_search_mode.set(false);
                                    set_search_results.set(None);
                                }
                            }
                        >
                            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
                            </svg>
                        </button>
                        {move || {
                            if search_mode.get() {
                                view! {
                                    <button
                                        class="clear-search-btn"
                                        on:click=move |_| {
                                            set_search_mode.set(false);
                                            set_search_query.set("".to_string());
                                            set_search_results.set(None);
                                        }
                                        title="Clear search"
                                    >
                                        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                                            <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
                                        </svg>
                                    </button>
                                }.into_any()
                            } else {
                                view! { <span></span> }.into_any()
                            }
                        }}
                    </div>

                    <button
                        class="toolbar-btn theme-btn"
                        on:click=toggle_theme
                        title=move || {
                            match theme.get().as_str() {
                                "light" => "Switch to Dark Mode",
                                _ => "Switch to Light Mode"
                            }
                        }
                    >
                        {move || {
                            match theme.get().as_str() {
                                "light" => view! {
                                    // Sun icon
                                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                        <circle cx="12" cy="12" r="5"/>
                                        <line x1="12" y1="1" x2="12" y2="3"/>
                                        <line x1="12" y1="21" x2="12" y2="23"/>
                                        <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/>
                                        <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/>
                                        <line x1="1" y1="12" x2="3" y2="12"/>
                                        <line x1="21" y1="12" x2="23" y2="12"/>
                                        <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/>
                                        <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
                                    </svg>
                                }.into_any(),
                                _ => view! {
                                    // Moon icon
                                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                        <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
                                    </svg>
                                }.into_any()
                            }
                        }}
                    </button>

                    <button class="toolbar-btn view-btn active">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M4 6H2v14c0 1.1.9 2 2 2h14v-2H4V6zm16-4H8c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2zm-1 9H9V9h10v2zm-4 4H9v-2h6v2zm4-8H9V5h10v2z"/>
                        </svg>
                    </button>
                </div>
            </div>

            <div class="main-content">
                // Sidebar
                <div class=move || format!("sidebar {}", if sidebar_collapsed.get() { "collapsed" } else { "" })>
                    <div class="sidebar-section">
                        <div class="sidebar-title">"Favorites"</div>
                        <div class="sidebar-item" on:click=move |_| {
                            spawn_local(async move {
                                if is_tauri_available() {
                                    match invoke("get_home_directory", JsValue::NULL).await {
                                        Ok(home_path_value) => {
                                            if let Some(home_path) = home_path_value.as_string() {
                                                navigate_to(home_path);
                                            }
                                        }
                                        Err(_) => {
                                            navigate_to("/Users/demo".to_string());
                                        }
                                    }
                                } else {
                                    navigate_to("/Users/demo".to_string());
                                }
                            });
                        }>
                            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z"/>
                            </svg>
                            <span>"Home"</span>
                        </div>
                        <div class="sidebar-item" on:click=move |_| navigate_to("/Applications".to_string())>
                            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M4 6H2v14c0 1.1.9 2 2 2h14v-2H4V6zm16-4H8c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2zm-1 9H9V9h10v2zm-4 4H9v-2h6v2zm4-8H9V5h10v2z"/>
                            </svg>
                            <span>"Applications"</span>
                        </div>
                        <div class="sidebar-item" on:click=move |_| navigate_to("/Users".to_string())>
                            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z"/>
                            </svg>
                            <span>"Users"</span>
                        </div>
                    </div>
                </div>

                // Content area with file list and preview panel
                <div class="content-area">
                    // File list area
                    <div
                        class="file-list-container"
                        style=move || format!("transform: scale({}); transform-origin: top left;", zoom_level.get())
                        on:wheel=move |e| {
                            // Check if Ctrl/Cmd key is pressed for zoom
                            if e.ctrl_key() || e.meta_key() {
                                e.prevent_default();

                                let delta_y = e.delta_y();
                                let current_zoom = zoom_level.get();

                                let new_zoom = if delta_y > 0.0 {
                                    // Zoom out
                                    (current_zoom - zoom_step).max(min_zoom)
                                } else {
                                    // Zoom in
                                    (current_zoom + zoom_step).min(max_zoom)
                                };

                                set_zoom_level.set(new_zoom);
                            }
                            // If no modifier keys, allow normal scrolling
                        }
                    >
                        {move || {
                        if searching.get() {
                            view! {
                                <div class="loading">
                                    <div class="loading-spinner"></div>
                                    <span>"Searching..."</span>
                                </div>
                            }.into_any()
                        } else if search_mode.get() {
                            // Search results view
                            if let Some(results) = search_results.get() {
                                view! {
                                    <div class="file-list">
                                        <div class="search-header">
                                            <h3>{format!("Search results for \"{}\" in {}", search_query.get(), current_path.get())}</h3>
                                            <p>{format!("{} items found", results.len())}</p>
                                        </div>
                                        <div class="file-list-header">
                                            <div class="file-header-name">"Name"</div>
                                            <div class="file-header-modified">"Date Modified"</div>
                                            <div class="file-header-size">"Size"</div>
                                        </div>
                                        <div class="file-list-body">
                                            {results.into_iter().map(|item| {
                                                let item_path = item.path.clone();
                                                let item_path_click = item_path.clone();
                                                let item_path_dblclick = item_path.clone();
                                                let item_path_context = item_path.clone();
                                                let item_path_focused = item_path.clone();
                                                let _item_name = item.name.clone();
                                                let is_dir = item.is_dir;
                                                view! {
                                                    <div
                                                        class="file-item"
                                                        class:selected=move || selected_item.get() == Some(item_path.clone())
                                                        class:focused=move || focused_item.get() == Some(item_path_focused.clone())
                                                        tabindex="0"
                                                        on:click=move |_| {
                                                            set_selected_item.set(Some(item_path_click.clone()));
                                                            set_context_menu_visible.set(false);
                                                        }
                                                        on:dblclick=move |_| {
                                                            if is_dir {
                                                                // Exit search mode and navigate to directory
                                                                set_search_mode.set(false);
                                                                set_search_results.set(None);
                                                                navigate_to(item_path_dblclick.clone());
                                                            }
                                                        }
                                                        on:contextmenu=move |e| {
                                                            e.prevent_default();
                                                            set_selected_item.set(Some(item_path_context.clone()));
                                                            set_context_menu_pos.set((e.client_x(), e.client_y()));
                                                            set_context_menu_visible.set(true);
                                                        }
                                                    >
                                                        <div class="file-item-name">
                                                            <FileIcon icon=item.icon.clone() />
                                                            <span class="file-name">{item.name}</span>
                                                            <span class="file-path">{item.path}</span>
                                                        </div>
                                                        <div class="file-item-modified">
                                                            {item.modified.unwrap_or_else(|| "--".to_string())}
                                                        </div>
                                                        <div class="file-item-size">
                                                            {if item.is_dir {
                                                                "--".to_string()
                                                            } else {
                                                                format_file_size(item.size.unwrap_or(0))
                                                            }}
                                                        </div>
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="empty">
                                        <svg width="48" height="48" viewBox="0 0 24 24" fill="currentColor">
                                            <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
                                        </svg>
                                        <h3>"No results found"</h3>
                                        <p>{format!("No files matching \"{}\" found in {}", search_query.get(), current_path.get())}</p>
                                    </div>
                                }.into_any()
                            }
                        } else if loading.get() {
                            view! {
                                <div class="loading">
                                    <div class="loading-spinner"></div>
                                    <span>"Loading..."</span>
                                </div>
                            }.into_any()
                        } else if let Some(error) = error_msg.get() {
                            view! {
                                <div class="error">
                                    <svg width="48" height="48" viewBox="0 0 24 24" fill="currentColor">
                                        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
                                    </svg>
                                    <h3>"Error"</h3>
                                    <p>{error}</p>
                                </div>
                            }.into_any()
                        } else if let Some(cols) = (!columns.get().is_empty()).then(|| columns.get()) {
                            // Multi-column view
                            view! {
                                <div class="columns-container">
                                    {cols.into_iter().enumerate().map(|(col_index, column)| {
                                        view! {
                                            <div class="file-column">
                                                <div class="file-list">
                                                    <div class="file-list-header">
                                                        <div class="file-header-name">"Name"</div>
                                                        <div class="file-header-modified">"Date Modified"</div>
                                                        <div class="file-header-size">"Size"</div>
                                                    </div>
                                                    <div class="file-list-body">
                                                        {column.contents.items.into_iter().map(|item| {
                                                            let item_path = item.path.clone();
                                                            let item_path_click = item_path.clone();
                                                            let item_path_dblclick = item_path.clone();
                                                            let item_path_context = item_path.clone();
                                                            let item_path_focused = item_path.clone();
                                                            let _item_name = item.name.clone();
                                                            let is_dir = item.is_dir;
                                                            let current_col_index = col_index;
                                                            view! {
                                                                <div
                                                                    class="file-item"
                                                                    class:selected=move || {
                                                                        selected_item.get() == Some(item_path.clone()) &&
                                                                        selected_column_index.get() == Some(current_col_index)
                                                                    }
                                                                    class:focused=move || {
                                                                        focused_item.get() == Some(item_path_focused.clone()) &&
                                                                        focused_column_index.get() == Some(current_col_index)
                                                                    }
                                                                    tabindex="0"
                                                                    on:click=move |_| {
                                                                        set_selected_item.set(Some(item_path_click.clone()));
                                                                        set_selected_column_index.set(Some(current_col_index));
                                                                        set_context_menu_visible.set(false);

                                                                        // If it's a file, truncate columns after current column and add preview
                                                                        if !is_dir {
                                                                            set_columns.update(|cols| {
                                                                                // Remove all columns after the current one
                                                                                cols.truncate(current_col_index + 1);
                                                                            });

                                                                            // Scroll to rightmost column (which will include the preview)
                                                                            scroll_to_rightmost_column();
                                                                        }
                                                                    }
                                                                    on:dblclick=move |_| {
                                                        if is_dir {
                                                            let new_col_index = current_col_index + 1;
                                                            load_directory_column(item_path_dblclick.clone(), Some(new_col_index));

                                                            // Set focus to the new column
                                                            set_focused_column_index.set(Some(new_col_index));

                                                            // Scroll to rightmost column
                                                            scroll_to_rightmost_column();

                                                            // Focus on first item in new column after it loads
                                                            spawn_local(async move {
                                                                // Wait a bit for the new column to load
                                                                wasm_bindgen_futures::JsFuture::from(
                                                                    js_sys::Promise::new(&mut |resolve, _| {
                                                                        web_sys::window()
                                                                            .unwrap()
                                                                            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 100)
                                                                            .unwrap();
                                                                    })
                                                                ).await.unwrap();

                                                                if let Some(new_column) = columns.get().get(new_col_index) {
                                                                    if !new_column.contents.items.is_empty() {
                                                                        let first_item = &new_column.contents.items[0];
                                                                        set_focused_item.set(Some(first_item.path.clone()));
                                                                        scroll_to_focused_item();
                                                                    }
                                                                }
                                                            });
                                                        }
                                                    }
                                                                    on:contextmenu=move |e| {
                                                                        e.prevent_default();
                                                                        set_selected_item.set(Some(item_path_context.clone()));
                                                                        set_selected_column_index.set(Some(current_col_index));
                                                                        set_context_menu_pos.set((e.client_x(), e.client_y()));
                                                                        set_context_menu_visible.set(true);
                                                                    }
                                                                >
                                                                    <div class="file-item-name">
                                                                        <FileIcon icon=item.icon.clone() />
                                                                        <span class="file-name">{item.name}</span>
                                                                    </div>
                                                                    <div class="file-item-modified">
                                                                        {item.modified.unwrap_or_else(|| "--".to_string())}
                                                                    </div>
                                                                    <div class="file-item-size">
                                                                        {if item.is_dir {
                                                                            "--".to_string()
                                                                        } else {
                                                                            format_file_size(item.size.unwrap_or(0))
                                                                        }}
                                                                    </div>
                                                                </div>
                                                            }
                                                        }).collect::<Vec<_>>()}
                                                    </div>
                                                </div>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}

                                    // Add preview column when a file is selected
                                    {move || {
                                        if let Some(selected_path) = selected_item.get() {
                                            // Check if the selected item is a file (not a directory)
                                            let is_file = if let Some(col_index) = selected_column_index.get() {
                                                if let Some(column) = columns.get().get(col_index) {
                                                    column.contents.items.iter()
                                                        .find(|item| item.path == selected_path)
                                                        .map(|item| !item.is_dir)
                                                        .unwrap_or(false)
                                                } else {
                                                    false
                                                }
                                            } else if let Some(search_results) = search_results.get() {
                                                search_results.iter()
                                                    .find(|item| item.path == selected_path)
                                                    .map(|item| !item.is_dir)
                                                    .unwrap_or(false)
                                            } else {
                                                false
                                            };

                                            // Only show preview column for files, not directories
                                            if is_file {
                                                view! {
                                                    <div class="column preview-column">
                                                        <div class="column-header">
                                                            <span class="column-title">"Preview"</span>
                                                        </div>
                                                        <div class="column-content">
                                                            <div class="preview-container">
                                                                {move || {
                                                                    // Get selected item info
                                                                    let selected_item_info = if let Some(col_index) = selected_column_index.get() {
                                                                        if let Some(column) = columns.get().get(col_index) {
                                                                            column.contents.items.iter()
                                                                                .find(|item| item.path == selected_path)
                                                                                .cloned()
                                                                        } else {
                                                                            None
                                                                        }
                                                                    } else if let Some(search_results) = search_results.get() {
                                                                        search_results.iter()
                                                                            .find(|item| item.path == selected_path)
                                                                            .cloned()
                                                                    } else {
                                                                        None
                                                                    };

                                                                    if let Some(item_info) = selected_item_info.clone() {
                                                                        if preview_loading.get() {
                                                                            view! {
                                                                                <div class="preview-loading">
                                                                                    <div class="loading-spinner"></div>
                                                                                    <span>"Loading preview..."</span>
                                                                                </div>
                                                                            }.into_any()
                                                                        } else if preview_error.get().is_some() {
                                                                             view! {
                                                                                 <div class="unsupported-preview">
                                                                                     <FileIcon icon=item_info.icon.clone() size="64".to_string() />
                                                                                 </div>
                                                                             }.into_any()
                                                                         } else if preview_content.get().is_some() {
                                                                             let preview = preview_content.get().unwrap();
                                                                             match preview.file_type.as_str() {
                                                                                 "text" => {
                                                                                     view! {
                                                                                         <div class="text-preview">
                                                                                             <pre class="text-content">{preview.content}</pre>
                                                                                         </div>
                                                                                     }.into_any()
                                                                                 }
                                                                                 "image" => {
                                                                                     view! {
                                                                                         <div class="image-preview">
                                                                                             <img
                                                                                                 src=format!("data:image/*;base64,{}", preview.content)
                                                                                                 alt="Preview"
                                                                                                 class="image-content"
                                                                                             />
                                                                                         </div>
                                                                                     }.into_any()
                                                                                 }
                                                                                 _ => {
                                                                                     view! {
                                                                                         <div class="unsupported-preview">
                                                                                             <FileIcon icon=item_info.icon.clone() size="64".to_string() />
                                                                                         </div>
                                                                                     }.into_any()
                                                                                 }
                                                                             }
                                                                         } else {
                                                                             view! {
                                                                                 <div class="preview-empty">
                                                                                     <FileIcon icon=item_info.icon.clone() size="64".to_string() />
                                                                                 </div>
                                                                             }.into_any()
                                                                         }
                                                                    } else {
                                                                        view! {
                                                                            <div class="preview-empty">
                                                                                <svg width="64" height="64" viewBox="0 0 24 24" fill="currentColor">
                                                                                    <path d="M14,2H6A2,2 0 0,0 4,4V20A2,2 0 0,0 6,22H18A2,2 0 0,0 20,20V8L14,2M18,20H6V4H13V9H18V20Z"/>
                                                                                </svg>
                                                                            </div>
                                                                        }.into_any()
                                                                    }
                                                                }}

                                                                // File info section
                                                                <div class="file-info">
                                                                    {move || {
                                                                        if let Some(selected_path) = selected_item.get() {
                                                                            let filename = selected_path.split('/').next_back().unwrap_or("").to_string();

                                                                            // Get item info for file details
                                                                            let item_info = if let Some(col_index) = selected_column_index.get() {
                                                                                if let Some(column) = columns.get().get(col_index) {
                                                                                    column.contents.items.iter()
                                                                                        .find(|item| item.path == selected_path)
                                                                                        .cloned()
                                                                                } else {
                                                                                    None
                                                                                }
                                                                            } else if let Some(search_results) = search_results.get() {
                                                                                search_results.iter()
                                                                                    .find(|item| item.path == selected_path)
                                                                                    .cloned()
                                                                            } else {
                                                                                None
                                                                            };

                                                                            view! {
                                                                                <div class="info-section">
                                                                                    <h4>{filename}</h4>
                                                                                    {move || {
                                                                                        if let Some(item) = item_info.clone() {
                                                                                            view! {
                                                                                                <div class="file-details">
                                                                                                    {if let Some(size) = item.size {
                                                                                                        view! {
                                                                                                            <div class="detail-item">
                                                                                                                <span class="label">"Size:"</span>
                                                                                                                <span class="value">{format_file_size(size)}</span>
                                                                                                            </div>
                                                                                                        }.into_any()
                                                                                                    } else {
                                                                                                        view! { <div></div> }.into_any()
                                                                                                    }}
                                                                                                    {if let Some(modified) = item.modified {
                                                                                                        view! {
                                                                                                            <div class="detail-item">
                                                                                                                <span class="label">"Modified:"</span>
                                                                                                                <span class="value">{modified}</span>
                                                                                                            </div>
                                                                                                        }.into_any()
                                                                                                    } else {
                                                                                                        view! { <div></div> }.into_any()
                                                                                                    }}
                                                                                                    <div class="detail-item">
                                                                                                        <span class="label">"Type:"</span>
                                                                                                        <span class="value">{"File"}</span>
                                                                                                    </div>
                                                                                                </div>
                                                                                            }.into_any()
                                                                                        } else if let Some(preview) = preview_content.get() {
                                                                                            view! {
                                                                                                <div class="file-details">
                                                                                                    <div class="detail-item">
                                                                                                        <span class="label">"Size:"</span>
                                                                                                        <span class="value">{format_file_size(preview.size)}</span>
                                                                                                    </div>
                                                                                                    {if preview.file_type == "text" {
                                                                                                        view! {
                                                                                                            <div class="detail-item">
                                                                                                                <span class="label">"Encoding:"</span>
                                                                                                                <span class="value">{preview.encoding}</span>
                                                                                                            </div>
                                                                                                        }.into_any()
                                                                                                    } else {
                                                                                                        view! { <div></div> }.into_any()
                                                                                                    }}
                                                                                                    <div class="detail-item">
                                                                                                        <span class="label">"Type:"</span>
                                                                                                        <span class="value">{preview.file_type}</span>
                                                                                                    </div>
                                                                                                </div>
                                                                                            }.into_any()
                                                                                        } else {
                                                                                            view! { <div></div> }.into_any()
                                                                                        }
                                                                                    }}
                                                                                </div>
                                                                            }.into_any()
                                                                        } else {
                                                                            view! { <div></div> }.into_any()
                                                                        }
                                                                    }}
                                                                </div>
                                                            </div>
                                                        </div>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! { <div></div> }.into_any()
                                            }
                                        } else {
                                            view! { <div></div> }.into_any()
                                        }
                                    }}
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="empty">
                                    <span>"No directory loaded"</span>
                                </div>
                            }.into_any()
                        }
                    }}
                </div>


            </div>

            // Rename dialog
            {move || {
                if show_rename_dialog.get() {
                    view! {
                        <div class="dialog-overlay" on:click=move |_| set_show_rename_dialog.set(false)>
                            <div class="dialog" on:click=move |e| e.stop_propagation()>
                                <h3>"Rename Item"</h3>
                                <input
                                    type="text"
                                    placeholder="New name"
                                    prop:value=move || rename_item_name.get()
                                    on:input=move |e| set_rename_item_name.set(event_target_value(&e))
                                />
                                <div class="dialog-buttons">
                                    <button on:click=move |_| set_show_rename_dialog.set(false)>
                                        "Cancel"
                                    </button>
                                    <button on:click=move |_| {
                                        let new_name = rename_item_name.get();
                                        let old_path = rename_item_path.get();
                                        if !new_name.is_empty() && !old_path.is_empty() {
                                            spawn_local(async move {
                                                rename_selected_item(old_path, new_name).await;
                                                refresh_current_column();
                                            });
                                            set_rename_item_name.set("".to_string());
                                            set_rename_item_path.set("".to_string());
                                            set_show_rename_dialog.set(false);
                                        }
                                    }>
                                        "Rename"
                                    </button>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }}
            </div>

            // Status bar
            <div class="status-bar">
                <div class="status-left">
                    {move || {
                        let cols = columns.get();
                        if let Some(last_col) = cols.last() {
                            format!("{} items", last_col.contents.items.len())
                        } else {
                            "".to_string()
                        }
                    }}
                </div>
                <div class="status-right">
                    {move || current_path.get()}
                </div>
            </div>

            // Context menu
            {move || {
                if context_menu_visible.get() {
                    let (x, y) = context_menu_pos.get();
                    view! {
                        <div
                            class="context-menu"
                            style=format!("left: {}px; top: {}px;", x, y)
                            on:click=move |_| set_context_menu_visible.set(false)
                        >
                            <div class="context-menu-item" on:click=move |_| {
                                if let Some(path) = selected_item.get() {
                                    set_clipboard_item.set(Some(path));
                                    set_clipboard_operation.set(Some("copy".to_string()));
                                }
                                set_context_menu_visible.set(false);
                            }>
                                "Copy"
                            </div>
                            <div class="context-menu-item" on:click=move |_| {
                                if let Some(path) = selected_item.get() {
                                    set_clipboard_item.set(Some(path));
                                    set_clipboard_operation.set(Some("cut".to_string()));
                                }
                                set_context_menu_visible.set(false);
                            }>
                                "Cut"
                            </div>
                            <div
                                class="context-menu-item"
                                class:disabled=move || clipboard_item.get().is_none()
                                on:click=move |_| {
                                     if let (Some(source_path), Some(operation)) = (clipboard_item.get(), clipboard_operation.get()) {
                                         let dest_dir = current_path.get();
                                         let operation_clone = operation.clone();
                                         spawn_local(async move {
                                             if operation_clone == "copy" {
                                                 let _ = copy_selected_item(source_path, dest_dir.clone()).await;
                                             } else if operation_clone == "cut" {
                                                 let _ = move_selected_item(source_path, dest_dir.clone()).await;
                                             }
                                             refresh_current_column();
                                         });
                                         if operation == "cut" {
                                             set_clipboard_item.set(None);
                                             set_clipboard_operation.set(None);
                                         }
                                     }
                                     set_context_menu_visible.set(false);
                                 }
                            >
                                "Paste"
                            </div>
                            <div class="context-menu-separator"></div>
                            <div class="context-menu-item" on:click=move |_| {
                                if let Some(path) = selected_item.get() {
                                    // Extract filename from path for initial value
                                    let filename = path.split('/').next_back().unwrap_or("").to_string();
                                    set_rename_item_name.set(filename);
                                    set_rename_item_path.set(path);
                                    set_show_rename_dialog.set(true);
                                }
                                set_context_menu_visible.set(false);
                            }>
                                "Rename"
                            </div>
                            <div class="context-menu-item" on:click=move |_| {
                                if let Some(path) = selected_item.get() {
                                    spawn_local(async move {
                                        delete_selected_item(path).await;
                                        refresh_current_column();
                                    });
                                }
                                set_context_menu_visible.set(false);
                            }>
                                "Delete"
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }}

            // New folder dialog
            {move || {
                if show_new_folder_dialog.get() {
                    view! {
                        <div class="dialog-overlay" on:click=move |_| set_show_new_folder_dialog.set(false)>
                            <div class="dialog" on:click=move |e| e.stop_propagation()>
                                <h3>"New Folder"</h3>
                                <input
                                    type="text"
                                    placeholder="Folder name"
                                    prop:value=move || new_folder_name.get()
                                    on:input=move |e| set_new_folder_name.set(event_target_value(&e))
                                />
                                <div class="dialog-buttons">
                                    <button on:click=move |_| set_show_new_folder_dialog.set(false)>
                                        "Cancel"
                                    </button>
                                    <button on:click=move |_| {
                                        let folder_name = new_folder_name.get();
                                        if !folder_name.is_empty() {
                                            spawn_local(async move {
                                                create_new_folder(current_path.get(), folder_name).await;
                                                refresh_current_column();
                                            });
                                            set_new_folder_name.set("".to_string());
                                            set_show_new_folder_dialog.set(false);
                                        }
                                    }>
                                        "Create"
                                    </button>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }}
        </div>

        // Click outside to close context menu
        <div
            class="click-overlay"
            style=format!("display: {}", if context_menu_visible.get() { "block" } else { "none" })
            on:click=move |_| set_context_menu_visible.set(false)
        ></div>
    }
}
