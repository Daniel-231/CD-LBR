// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod cd; // I guess import in rust

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
/*
#[tauri::command]
fn list_drives() -> Result<Vec<String>, String>{
    let drives = CdReader::list_drives().map_err(|e| e.to_string())?;

    let mut paths = Vec::new();
    for drive in &drives {
        paths.push(drive.path.clone());
    }
    Ok(paths)
}
*/

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, cd::list_drives, cd::list_track_nums, cd::preview_track, cd::play_selected_track])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
