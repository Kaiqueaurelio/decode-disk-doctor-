#[tauri::command]
fn list_storage_devices() -> Result<Vec<decode_core::DeviceIdentity>, String> {
    device_discovery::discover_devices().map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![list_storage_devices])
        .run(tauri::generate_context!())
        .expect("error while running Decode Disk Doctor");
}
