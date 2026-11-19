use cd_da_reader::CdReader; // for reading CD-DA audio tracks
use std::num::NonZero; // for non-zero integers
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError}; // for message passing
use std::thread; // for threading  
use std::time::Duration; // for time


use rodio::buffer::SamplesBuffer; // for audio buffer
use rodio::{DeviceSinkBuilder, Player, MixerDeviceSink}; // for audio playback
use tauri::State; // for state management


const SAMPLE_RATE: NonZero<u32> = NonZero::new(44_100).unwrap(); // 44.1 kHz
const CHANNELS: NonZero<u16> = NonZero::new(2).unwrap(); // stereo
const CHUNKS_AHEAD: usize = 3;

enum Command {
    Play { path: String, track: u8 },
    Stop,
    Pause,
    Resume,
    Seek(f32)
}

pub struct PlayerHandle {
    sender: Sender<Command>,
}

fn bytes_to_buffer(bytes: &[u8]) -> SamplesBuffer {
    let mut samples: Vec<f32> = Vec::with_capacity(bytes.len() / 2);

    // chunks_exact(2) hands us the bytes two at a time: [low, high]
    for pair in bytes.chunks_exact(2) {
        let value: i16 = i16::from_le_bytes([pair[0], pair[1]]);
        let as_float: f32 = value as f32 / 32768.0;
        samples.push(as_float);
    }

    SamplesBuffer::new(CHANNELS, SAMPLE_RATE, samples)
}

fn play_one_track(device: &MixerDeviceSink, receiver: &Receiver<Command>, path: &str, track: u8) -> Result<(), String> {
    // 1. Open the drive and read the table of contents
    let reader = CdReader::open_path(path).map_err(|e| format!("Failed to open CD-DA file: {}", e))?;
    let toc = reader.read_toc().map_err(|e| format!("Failed to read TOC: {}", e))?;

    // 2. Open a stream for this track. 75 sectors = 1 second of audio per chunk.
    let stream_result = reader.open_track_stream(&toc, track);
    let stream = stream_result.map_err(|e| format!("Failed to open track stream: {}", e))?;
    let mut stream = stream.with_sectors_per_chunk(75);

    // 3. Player connected to the speakers
    let player = Player::connect_new(device.mixer());

    let mut done_reading = false;

    // 4. Feed loop: keep CHUNKS_AHEAD chunks queued
    loop {
        // A. Peek at the mailbox. try_recv() never waits.
        match receiver.try_recv() {
            Ok(Command::Stop) => {
                player.stop(); // silence right now
                return Ok(());
            }
            Ok(Command::Pause) => {
                player.pause();
            }
            Ok(Command::Resume) => {
                player.play();
            }
            Ok(Command::Seek(position)) => {
                if position > 0.0 {
                    player.try_seek(Duration::from_secs_f32(position)).map_err(|e| format!("Failed to seek: {}", e))?;
                } else {
                    player.try_seek(Duration::from_secs_f32(-position)).map_err(|e| format!("Failed to seek: {}", e))?;
                }
            }
            Ok(_) => {
                // Play / Pause / Seek while playing: not handled yet
            }
            Err(TryRecvError::Empty) => {
                // no letters, keep playing
            }
            Err(TryRecvError::Disconnected) => {
                return Ok(()); // app is closing
            }
        }

        // B. Feed, finish, or rest
        if !done_reading && player.len() < CHUNKS_AHEAD {
            let next = stream.next_chunk().map_err(|e| e.to_string())?;
            match next {
                Some(bytes) => {
                    let buffer = bytes_to_buffer(&bytes);
                    player.append(buffer);
                }
                None => {
                    done_reading = true; // disc part done, but ~3 s still queued
                }
            }
        } else if done_reading && player.empty() {
            break; // read everything AND played everything
        } else {
            thread::sleep(Duration::from_millis(20));
        }
    }

    Ok(())

}

// The player thread's main loop. Owns the speakers for the life of the app.
fn player_loop(receiver: Receiver<Command>) {
    // Open the speakers ONCE. `device` must stay alive the whole time.
    let device = match DeviceSinkBuilder::open_default_sink() {
        Ok(device) => device,
        Err(e) => {
            eprintln!("player: could not open speakers: {e}");
            return;
        }
    };

    loop {
        // recv() waits until a command arrives.
        // It returns Err only when every Sender is gone (the app is closing).
        let command = match receiver.recv() {
            Ok(command) => command,
            Err(_) => break,
        };

        match command {
            Command::Play { path, track } => {
                if let Err(e) = play_one_track(&device, &receiver, &path, track) {
                    eprintln!("player: {e}");
                }
            }
            _ => {
                
            }
        }
    }
}

// Start the player thread and give back the sending end of the channel.
pub fn spawn_player() -> PlayerHandle {
    let (sender, receiver) = mpsc::channel::<Command>();

    thread::spawn(move || {
        player_loop(receiver);
    });

    PlayerHandle { sender }
}

// Called from React: invoke("play_track", { path, track })
#[tauri::command]
pub fn play_track(path: String, track: u8, handle: State<'_, PlayerHandle>) -> Result<(), String> {
    let command = Command::Play { path, track };
    handle.sender.send(command).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn pause_track(handle: State<'_, PlayerHandle>) -> Result<(), String> {
    handle.sender.send(Command::Pause).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn stop_track(handle: State<'_, PlayerHandle>) -> Result<(), String> {
    handle.sender.send(Command::Stop).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn resume_track(handle: State<'_, PlayerHandle>) -> Result<(), String> {
    handle.sender.send(Command::Resume).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn seek_track(position: f32, handle: State<'_, PlayerHandle>) -> Result<(), String> {
    handle.sender.send(Command::Seek(position)).map_err(|e| e.to_string())
}