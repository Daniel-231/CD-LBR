use cd_da_reader::{CdReader, ReadOptions, create_wav};
use tauri::ipc::Response;


#[tauri::command]
pub fn list_drives() -> Result<Vec<String>, String> {
    let drives = CdReader::list_drives().map_err(|e| e.to_string())?;
    let mut paths = Vec::new();
    for drive in &drives {
        paths.push(drive.path.clone());
    }
    Ok(paths)
}   

#[tauri::command]
// Get the track numbers from the TOC
pub fn list_track_nums(path: String) -> Result<Vec<u8>, String>{ // For now the drive path will be the first one by default (from the client)
    let reader = CdReader::open_path(&path).map_err(|e| e.to_string())?;
    let toc = reader.read_toc().map_err(|e| e.to_string())?; // Table Of Contennts

    let mut track_nums = Vec::new();
    for track in &toc.tracks {
        track_nums.push(track.number);
    }

    Ok(track_nums)
}

#[tauri::command]
pub fn preview_track(path: String) -> Result<Response, String> {
    let reader = CdReader::open_path(&path).map_err(|e| e.to_string())?;
    let toc = reader.read_toc().map_err(|e| e.to_string())?;

    let track = toc.tracks.iter().find(|t| t.is_audio).ok_or("No Tracks Found")?;

    let sectors= 75 * 30; // 75 sectors = 1 second of audio, so this is 30 seconds
    let pcm = reader.read_sector_range(track.start_lba, sectors, &ReadOptions::default()).map_err(|e| e.to_string())?;

    Ok(Response::new(create_wav(pcm)))
    
}

#[tauri::command]
pub fn play_selected_track(path: String, track_number: u8) -> Result<Response, String> {
    let reader = CdReader::open_path(&path).map_err(|e| e.to_string())?;
    let toc = reader.read_toc().map_err(|e| e.to_string())?;

    let pcm = reader.read_track(&toc, track_number).map_err(|e| e.to_string())?;

    Ok(Response::new(create_wav(pcm)))
}