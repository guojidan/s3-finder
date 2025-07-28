use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    pub async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

// Check if we're running in Tauri environment
pub fn is_tauri_available() -> bool {
    use wasm_bindgen::JsValue;
    
    // Safe window access
    let window = match web_sys::window() {
        Some(w) => w,
        None => return false,
    };
    
    let tauri = js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__"));
    match tauri {
        Ok(value) => !value.is_null() && !value.is_undefined(),
        Err(_) => false,
    }
}