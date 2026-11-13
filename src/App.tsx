import { useEffect, useState, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

function App() {
  const [drives, setDrives] = useState<string[]>([]);
  const [trackNums, setTrackNums] = useState<Number[]>([]);
  const [error, setError] = useState("");

  const audioRef = useRef<HTMLAudioElement | null>(null);

  useEffect(() => {
    invoke<string[]>("list_drives")
      .then(setDrives)
      .catch((e) => setError(String(e)));
  }, []);

  useEffect(() => {
    if (drives.length === 0) return;

    invoke<number[]>("list_track_nums", {path: drives[0]}).then(setTrackNums).catch((e) => setError(String(e)));

  }, [drives]);

  // Play the selected track
  async function playTrack(track_number: number) { // Rust reads CD  →  bytes  →  Blob  →  URL  →  Audio  →  sound
    try {
      if (audioRef.current) { // If there is an audio element playing, pause it and revoke the object URL
        audioRef.current.pause();
        URL.revokeObjectURL(audioRef.current.src); // Free the memory allocated to the object URL
      }
      const bytes = await invoke<ArrayBuffer>("play_selected_track", { path: drives[0], trackNumber: track_number }); // Get the audio data for the selected track
      const url = URL.createObjectURL(new Blob([bytes], { type: "audio/wav" })); // Create a URL for the audio data
      const audio = new Audio(url); // Create a new audio element
      audioRef.current = audio; // Set the audio element to the current audio element
      audio.play(); // Play the audio
    } catch (error) {
      setError(String(error));
    }
  }

  // Pause the audio
  function stopTrack() {
    if (audioRef.current) {
      audioRef.current.pause();
      URL.revokeObjectURL(audioRef.current.src);
    }
  }

  // Continiue Playing Track
  function continuePlayingTrack() {
    if (audioRef.current) {
      audioRef.current.play();
    }
  }

  // Forward Track
  function forwardTrack() {
    if (audioRef.current) {
      audioRef.current.currentTime += 10;
    }
  }

  // Rewind Track
  function rewindTrack() {
    if (audioRef.current) {
      audioRef.current.currentTime -= 10;
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
      <button onClick={stopTrack}>Stop</button>
      <button onClick={forwardTrack}>+10s</button>
      <button onClick={rewindTrack}>-10s</button>
      <button onClick={continuePlayingTrack}>Continue</button>
    </main>
  );
}

export default App;