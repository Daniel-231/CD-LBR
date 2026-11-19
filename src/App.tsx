import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

function App() {
  const [drives, setDrives] = useState<string[]>([]);
  const [trackNums, setTrackNums] = useState<Number[]>([]);
  const [error, setError] = useState("");

  useEffect(() => {
    invoke<string[]>("list_drives")
      .then(setDrives)
      .catch((e) => setError(String(e)));
  }, []);

  useEffect(() => {
    if (drives.length === 0) return;

    invoke<number[]>("list_track_nums", {path: drives[0]}).then(setTrackNums).catch((e) => setError(String(e)));

  }, [drives]);

  // Play the selected track through the Rust player (rodio)
  async function playTrack(track_number: number) {
    try {
      await invoke("play_track", { path: drives[0], track: track_number });
    } catch (error) {
      setError(String(error));
    }
  }

  // Stop the track (Rust player)
  async function stopTrack() {
    try {
      await invoke("stop_track");
    } catch (error) {
      setError(String(error));
    }
  }

  // Pause the track (Rust player)
  async function pauseTrack() {
    try {
      await invoke("pause_track");
    } catch (error) {
      setError(String(error));
    }
  }

  // Resume a paused track (Rust player)
  async function resumeTrack() {
    try {
      await invoke("resume_track");
    } catch (error) {
      setError(String(error));
    }
  }

  // Seek to a specific position in the track (Rust player)
  async function seekTrack(position: number) {
    try {
      await invoke("seek_track", { position });
    } catch (error) {
      setError(String(error));
    }
  }
  return (
    <main>
      <h1>CD drives</h1>

      {error && <p>Error: {error}</p>}

      {drives.length === 0 && !error && <p>No drives found.</p>}

      <ul>
        {drives.map((drive) => (
          <li key={drive}>{drive}</li>
        ))}
      </ul>

      <h2>Tracks</h2>

      <ul>
        {trackNums.map((n: any) => ( 
          <li key={n}>Track {n} <button onClick={() => playTrack(n)}>Play</button></li>
        ))}
      </ul>
      <button onClick={pauseTrack}>Pause</button>
      <button onClick={resumeTrack}>Resume</button>
      <button onClick={stopTrack}>Stop</button>
      <button onClick={() => seekTrack(10)}>Seek to 10 seconds</button>
      <button onClick={() => seekTrack(-10)}>Back 10 seconds</button>
    </main>
  );
}

export default App;
