#[allow(unused_imports)]
use std::ptr;
#[allow(unused_imports)]
use std::mem;
use winapi::um::winuser::{GetForegroundWindow, GetWindowTextW, GetWindowTextLengthW};
use winapi::um::winnt::WCHAR;
use winapi::um::winuser::{GetAsyncKeyState, VK_OEM_PLUS, VK_CONTROL, VK_SHIFT};
use willhook::keyboard_hook;
use std::sync::{Arc, atomic::{Ordering, AtomicBool}};

fn main() {
    let is_running = Arc::new(AtomicBool::new(true));
    let set_running = is_running.clone();

    let h = keyboard_hook().unwrap();

    ctrlc::set_handler(move || {
        set_running.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    unsafe {
        let mut last_title = String::new(); // Store the last active window title
        loop {
            let ctrl_pressed = GetAsyncKeyState(VK_CONTROL) & 0x8000u16 as i16 != 0;
            let shift_pressed = GetAsyncKeyState(VK_SHIFT) & 0x8000u16 as i16 != 0;
            let plus_pressed = GetAsyncKeyState(VK_OEM_PLUS) & 0x8000u16 as i16 != 0;

            if ctrl_pressed && shift_pressed && plus_pressed {
                println!("Ctrl+Shift+Plus pressed. Exiting...");
                break;
            }

            let hwnd = GetForegroundWindow();
            let mut buffer: Vec<WCHAR> = vec![0; 256]; // Adjust the buffer size as needed
            let len = GetWindowTextLengthW(hwnd);
            GetWindowTextW(hwnd, buffer.as_mut_ptr(), len + 1);
            let title = String::from_utf16_lossy(&buffer);

            if let Ok(ie) = h.try_recv() {
                match ie {
                    willhook::InputEvent::Keyboard(ke) => {
                        if let Some(key) = ke.key {
                            if title != last_title {
                                println!("{} : {:?}",title, key);
                                last_title = title.clone();
                            }
                            else {
                                println!("{} : {:?}",last_title, key);
                            }
                        } else {
                            println!("Input event: {:?}", ie);
                        }
                    }
                    _ => println!("Input event: {:?}", ie),
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}
