use serde::{Deserialize, Serialize};
use midly::{Smf, TrackEventKind, MidiMessage, MetaMessage};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
use tauri::Emitter; 
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[cfg(windows)]
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_KEYBOARD, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, 
    VkKeyScanW, MapVirtualKeyW, MAPVK_VK_TO_VSC
};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct MidiNote {
    note: u8,
    velocity: u8,
    time_ms: f64,
    duration_ms: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct MidiTrack {
    index: usize,
    name: String,
    note_count: usize,
    notes: Vec<MidiNote>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MidiFileInfo {
    file_name: String,
    duration_ms: f64,
    tempo_bpm: f64,
    tracks: Vec<MidiTrack>,
}

struct ClickerState {
    is_running: bool,
    thread_handle: Option<thread::JoinHandle<()>>,
}

lazy_static::lazy_static! {
    static ref CLICKER_STATE: Arc<Mutex<ClickerState>> = Arc::new(Mutex::new(ClickerState {
        is_running: false,
        thread_handle: None,
    }));
}

#[derive(Serialize, Deserialize, Debug)]
struct NoteEvent {
    key: String,
    time_ms: u64,
    duration_ms: u64,  // Добавили длительность
}

#[cfg(windows)]
fn press_key_native(key_char: char) {
    unsafe {
        let ch_upper = key_char.to_ascii_uppercase();
        let vk_code = VkKeyScanW(ch_upper as u16);
        let vk = (vk_code & 0xFF) as u16;
        let scan = MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC) as u16;
        
        // Только нажатие (без release)
        let input = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                ki: windows::Win32::UI::Input::KeyboardAndMouse::KEYBDINPUT {
                    wVk: windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY(0),
                    wScan: scan,
                    dwFlags: KEYEVENTF_SCANCODE,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        
        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
    }
}

#[cfg(windows)]
fn release_key_native(key_char: char) {
    unsafe {
        let ch_upper = key_char.to_ascii_uppercase();
        let vk_code = VkKeyScanW(ch_upper as u16);
        let vk = (vk_code & 0xFF) as u16;
        let scan = MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC) as u16;
        
        // Только отпускание
        let input = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                ki: windows::Win32::UI::Input::KeyboardAndMouse::KEYBDINPUT {
                    wVk: windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY(0),
                    wScan: scan,
                    dwFlags: KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        
        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
    }
}

#[tauri::command]
fn start_clicker(notes: Vec<NoteEvent>) -> Result<(), String> {
    println!("Starting clicker with {} notes", notes.len());
    
    let mut state = CLICKER_STATE.lock().unwrap();
    
    if state.is_running {
        return Err("Clicker is already running".to_string());
    }
    
    state.is_running = true;
    let clicker_state = Arc::clone(&CLICKER_STATE);
    
    let handle = thread::spawn(move || {
        println!("Clicker thread started");
        let start_time = std::time::Instant::now();
        
        for note in notes.iter() {
            {
                let state = clicker_state.lock().unwrap();
                if !state.is_running {
                    println!("Clicker stopped early");
                    break;
                }
            }
            
            // Ждем до времени ноты
            let target_time = Duration::from_millis(note.time_ms);
            let elapsed = start_time.elapsed();
            
            if target_time > elapsed {
                thread::sleep(target_time - elapsed);
            }
            
            println!("Pressing key '{}' at {}ms for {}ms", note.key, note.time_ms, note.duration_ms);

            #[cfg(windows)]
            {
                if let Some(ch) = note.key.chars().next() {
                    // Нажимаем клавишу
                    press_key_native(ch);
                    
                    // Держим нажатой
                    thread::sleep(Duration::from_millis(note.duration_ms));
                    
                    // Отпускаем
                    release_key_native(ch);
                }
            }
        }
        
        println!("Clicker finished");
        let mut state = clicker_state.lock().unwrap();
        state.is_running = false;
    });
    
    state.thread_handle = Some(handle);
    Ok(())
}

#[tauri::command]
fn stop_clicker() -> Result<(), String> {
    let mut state = CLICKER_STATE.lock().unwrap();
    
    if !state.is_running {
        return Ok(());
    }
    
    state.is_running = false;
    
    Ok(())
}

#[tauri::command]
fn is_clicker_running() -> bool {
    let state = CLICKER_STATE.lock().unwrap();
    state.is_running
}

#[tauri::command]
fn parse_midi_data(file_name: String, data: Vec<u8>) -> Result<MidiFileInfo, String> {
    let smf = Smf::parse(&data)
        .map_err(|e| format!("Failed to parse MIDI: {}", e))?;
    
    let ticks_per_beat = match smf.header.timing {
        midly::Timing::Metrical(tpb) => tpb.as_int() as f64,
        _ => return Err("Timecode timing not supported".to_string()),
    };
    
    let mut tempo = 500000.0;
    let mut tracks = Vec::new();
    let mut max_time_ms: f64 = 0.0;
    
    for (track_index, track) in smf.tracks.iter().enumerate() {
        let mut current_ticks: u64 = 0;
        let mut current_tempo = tempo;
        let mut track_name = format!("Track {}", track_index + 1);
        let mut notes = Vec::new();
        let mut active_notes: std::collections::HashMap<u8, (u64, u8)> = std::collections::HashMap::new();
        
        let ticks_to_ms = |ticks: u64, tempo_val: f64| -> f64 {
            (ticks as f64 / ticks_per_beat) * (tempo_val / 1000.0)
        };
        
        for event in track {
            current_ticks += event.delta.as_int() as u64;
            
            match &event.kind {
                TrackEventKind::Midi { message, .. } => {
                    match message {
                        MidiMessage::NoteOn { key, vel } => {
                            let note = key.as_int();
                            let velocity = vel.as_int();
                            
                            if velocity > 0 {
                                active_notes.insert(note, (current_ticks, velocity));
                            } else {
                                if let Some((start_ticks, vel)) = active_notes.remove(&note) {
                                    let time_ms = ticks_to_ms(start_ticks, current_tempo);
                                    let duration_ms = ticks_to_ms(current_ticks - start_ticks, current_tempo);
                                    
                                    notes.push(MidiNote {
                                        note,
                                        velocity: vel,
                                        time_ms,
                                        duration_ms,
                                    });
                                    
                                    max_time_ms = max_time_ms.max(time_ms + duration_ms);
                                }
                            }
                        }
                        MidiMessage::NoteOff { key, .. } => {
                            let note = key.as_int();
                            if let Some((start_ticks, vel)) = active_notes.remove(&note) {
                                let time_ms = ticks_to_ms(start_ticks, current_tempo);
                                let duration_ms = ticks_to_ms(current_ticks - start_ticks, current_tempo);
                                
                                notes.push(MidiNote {
                                    note,
                                    velocity: vel,
                                    time_ms,
                                    duration_ms,
                                });
                                
                                max_time_ms = max_time_ms.max(time_ms + duration_ms);
                            }
                        }
                        _ => {}
                    }
                }
                TrackEventKind::Meta(meta) => {
                    match meta {
                        MetaMessage::Tempo(t) => {
                            current_tempo = t.as_int() as f64;
                            tempo = current_tempo;
                        }
                        MetaMessage::TrackName(name) => {
                            if let Ok(name_str) = std::str::from_utf8(name) {
                                if !name_str.is_empty() {
                                    track_name = name_str.to_string();
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        
        if !notes.is_empty() {
            notes.sort_by(|a, b| a.time_ms.partial_cmp(&b.time_ms).unwrap());
            
            tracks.push(MidiTrack {
                index: track_index,
                name: track_name,
                note_count: notes.len(),
                notes,
            });
        }
    }
    
    let tempo_bpm = 60000000.0 / tempo;
    
    Ok(MidiFileInfo {
        file_name,
        duration_ms: max_time_ms,
        tempo_bpm,
        tracks,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        let _ = app.emit("toggle-clicker", ());
                    }
                })
                .build()
        )
        .setup(|app| {
            let shortcut = "CommandOrControl+M".parse::<Shortcut>()?;
            app.global_shortcut().register(shortcut)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            parse_midi_data,
            start_clicker,
            stop_clicker,
            is_clicker_running
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}