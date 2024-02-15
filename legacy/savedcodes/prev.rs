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

    while is_running.load(Ordering::SeqCst) {
        if let Ok(ie) = h.try_recv() {
            match ie {
                willhook::InputEvent::Keyboard(ke) => println!("{:?}", ke.key),
                willhook::InputEvent::Mouse(me) => println!("{:?}", me),
                _ => println!("Input event: {:?}", ie),
            }
        } else {
            std::thread::yield_now();   
        }
    };
}