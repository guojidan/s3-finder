use leptos::prelude::*;

#[component]
pub fn FileIcon(icon: String, #[prop(default = "16".to_string())] size: String) -> impl IntoView {
    match icon.as_str() {
        "folder" => view! {
            <svg width=size.clone() height=size.clone() viewBox="0 0 24 24" fill="#007AFF">
                <path d="M10 4H4c-1.11 0-2 .89-2 2v12c0 1.11.89 2 2 2h16c1.11 0 2-.89 2-2V8c0-1.11-.89-2-2-2h-8l-2-2z"/>
            </svg>
        },
        "document-text" => view! {
            <svg width=size.clone() height=size.clone() viewBox="0 0 24 24" fill="#666">
                <path d="M14,2H6A2,2 0 0,0 4,4V20A2,2 0 0,0 6,22H18A2,2 0 0,0 20,20V8L14,2M18,20H6V4H13V9H18V20Z"/>
            </svg>
        },
        "document" => view! {
            <svg width=size.clone() height=size.clone() viewBox="0 0 24 24" fill="#666">
                <path d="M14,2H6A2,2 0 0,0 4,4V20A2,2 0 0,0 6,22H18A2,2 0 0,0 20,20V8L14,2M18,20H6V4H13V9H18V20Z"/>
            </svg>
        },
        "photo" => view! {
            <svg width=size.clone() height=size.clone() viewBox="0 0 24 24" fill="#FF9500">
                <path d="M21,19V5C21,3.89 20.1,3 19,3H5A2,2 0 0,0 3,5V19A2,2 0 0,0 5,21H19A2,2 0 0,0 21,19M19,19H5V5H19V19Z"/>
            </svg>
        },
        "code-bracket" => view! {
            <svg width=size.clone() height=size.clone() viewBox="0 0 24 24" fill="#34C759">
                <path d="M8,3A2,2 0 0,0 6,5V9A2,2 0 0,1 4,11H3V13H4A2,2 0 0,1 6,15V19A2,2 0 0,0 8,21H10V19H8V14A2,2 0 0,0 6,12A2,2 0 0,0 8,10V5H10V3M16,3A2,2 0 0,1 18,5V9A2,2 0 0,0 20,11H21V13H20A2,2 0 0,0 18,15V19A2,2 0 0,1 16,21H14V19H16V14A2,2 0 0,1 18,12A2,2 0 0,1 16,10V5H14V3H16Z"/>
            </svg>
        },
        "table" => view! {
            <svg width=size.clone() height=size.clone() viewBox="0 0 24 24" fill="#34C759">
                <path d="M3,3H21A2,2 0 0,1 23,5V19A2,2 0 0,1 21,21H3A2,2 0 0,1 1,19V5A2,2 0 0,1 3,3M5,5V7H11V5H5M13,5V7H19V5H13M5,9V11H11V9H5M13,9V11H19V9H13M5,13V15H11V13H5M13,13V15H19V13H13M5,17V19H11V17H5M13,17V19H19V17H13Z"/>
            </svg>
        },
        "presentation" => view! {
            <svg width=size.clone() height=size.clone() viewBox="0 0 24 24" fill="#FF9500">
                <path d="M2,3H10A2,2 0 0,1 12,1A2,2 0 0,1 14,3H22V5H21V16A2,2 0 0,1 19,18H5A2,2 0 0,1 3,16V5H2V3M5,5V16H19V5H5M6,6H18V7H6V6M6,8H18V9H6V8M6,10H18V11H6V10M6,12H18V13H6V12M6,14H18V15H6V14Z"/>
            </svg>
        },
        "film" => view! {
            <svg width=size.clone() height=size.clone() viewBox="0 0 24 24" fill="#FF3B30">
                <path d="M18,4L20,8H17L15,4H13L15,8H12L10,4H8L10,8H7L5,4H4A2,2 0 0,0 2,6V18A2,2 0 0,0 4,20H20A2,2 0 0,0 22,18V6A2,2 0 0,0 20,4H18Z"/>
            </svg>
        },
        "musical-note" => view! {
            <svg width=size.clone() height=size.clone() viewBox="0 0 24 24" fill="#FF9500">
                <path d="M12,3V13.55C11.41,13.21 10.73,13 10,13A4,4 0 0,0 6,17A4,4 0 0,0 10,21A4,4 0 0,0 14,17V7H18V3H12Z"/>
            </svg>
        },
        "archive-box" => view! {
            <svg width=size.clone() height=size.clone() viewBox="0 0 24 24" fill="#8E8E93">
                <path d="M12,2A2,2 0 0,1 14,4C14,4.74 13.6,5.39 13,5.73V7H14A7,7 0 0,1 21,14H22A1,1 0 0,1 23,15V18A1,1 0 0,1 22,19H21A7,7 0 0,1 14,26H10A7,7 0 0,1 3,19H2A1,1 0 0,1 1,18V15A1,1 0 0,1 2,14H3A7,7 0 0,1 10,7H11V5.73C10.4,5.39 10,4.74 10,4A2,2 0 0,1 12,2M12,4A0,0 0 0,0 12,4A0,0 0 0,0 12,4M10,9A5,5 0 0,0 5,14V17A5,5 0 0,0 10,22H14A5,5 0 0,0 19,17V14A5,5 0 0,0 14,9H10Z"/>
            </svg>
        },
        "cog" => view! {
            <svg width=size.clone() height=size.clone() viewBox="0 0 24 24" fill="#8E8E93">
                <path d="M12,15.5A3.5,3.5 0 0,1 8.5,12A3.5,3.5 0 0,1 12,8.5A3.5,3.5 0 0,1 15.5,12A3.5,3.5 0 0,1 12,15.5M19.43,12.97C19.47,12.65 19.5,12.33 19.5,12C19.5,11.67 19.47,11.34 19.43,11L21.54,9.37C21.73,9.22 21.78,8.95 21.66,8.73L19.66,5.27C19.54,5.05 19.27,4.96 19.05,5.05L16.56,6.05C16.04,5.66 15.5,5.32 14.87,5.07L14.5,2.42C14.46,2.18 14.25,2 14,2H10C9.75,2 9.54,2.18 9.5,2.42L9.13,5.07C8.5,5.32 7.96,5.66 7.44,6.05L4.95,5.05C4.73,4.96 4.46,5.05 4.34,5.27L2.34,8.73C2.22,8.95 2.27,9.22 2.46,9.37L4.57,11C4.53,11.34 4.5,11.67 4.5,12C4.5,12.33 4.53,12.65 4.57,12.97L2.46,14.63C2.27,14.78 2.22,15.05 2.34,15.27L4.34,18.73C4.46,18.95 4.73,19.03 4.95,18.95L7.44,17.94C7.96,18.34 8.5,18.68 9.13,18.93L9.5,21.58C9.54,21.82 9.75,22 10,22H14C14.25,22 14.46,21.82 14.5,21.58L14.87,18.93C15.5,18.68 16.04,18.34 16.56,17.94L19.05,18.95C19.27,19.03 19.54,18.95 19.66,18.73L21.66,15.27C21.78,15.05 21.73,14.78 21.54,14.63L19.43,12.97Z"/>
            </svg>
        },
        _ => view! {
            <svg width=size.clone() height=size.clone() viewBox="0 0 24 24" fill="#666">
                <path d="M14,2H6A2,2 0 0,0 4,4V20A2,2 0 0,0 6,22H18A2,2 0 0,0 20,20V8L14,2M18,20H6V4H13V9H18V20Z"/>
            </svg>
        },
    }
}