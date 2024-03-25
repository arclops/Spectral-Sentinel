use std::{
    collections::{HashMap, HashSet}, fs::File, io::{BufRead, BufReader}, sync::{mpsc::{self, TryRecvError},Arc, Mutex}, thread,
    time::Duration,
};
use tinyfiledialogs::open_file_dialog;
use willhook::KeyboardKey;
use crossterm::{execute, style::{Print, SetForegroundColor, Color, ResetColor}};

fn init_keymapper() -> HashMap<KeyboardKey, &'static str> {
    let mut map = std::collections::HashMap::new();
    map.insert(KeyboardKey::A, "a");
    map.insert(KeyboardKey::B, "b");
    map.insert(KeyboardKey::C, "c");
    map.insert(KeyboardKey::D, "d");
    map.insert(KeyboardKey::E, "e");
    map.insert(KeyboardKey::F, "f");
    map.insert(KeyboardKey::G, "g");
    map.insert(KeyboardKey::H, "h");
    map.insert(KeyboardKey::I, "i");
    map.insert(KeyboardKey::J, "j");
    map.insert(KeyboardKey::K, "k");
    map.insert(KeyboardKey::L, "l");
    map.insert(KeyboardKey::M, "m");
    map.insert(KeyboardKey::N, "n");
    map.insert(KeyboardKey::O, "o");
    map.insert(KeyboardKey::P, "p");
    map.insert(KeyboardKey::Q, "q");
    map.insert(KeyboardKey::R, "r");
    map.insert(KeyboardKey::S, "s");
    map.insert(KeyboardKey::T, "t");
    map.insert(KeyboardKey::U, "u");
    map.insert(KeyboardKey::V, "v");
    map.insert(KeyboardKey::W, "w");
    map.insert(KeyboardKey::X, "x");
    map.insert(KeyboardKey::Y, "y");
    map.insert(KeyboardKey::Z, "z");
    map.insert(KeyboardKey::Space, " ");
    map.insert(KeyboardKey::Enter, "\n");
    map.insert(KeyboardKey::BackSpace, "\x08");
    map.insert(KeyboardKey::Tab, "\t");
    map.insert(KeyboardKey::Escape, "\x1b");
    map.insert(KeyboardKey::Delete, "\x7f");
    map.insert(KeyboardKey::ArrowLeft, "\x1b[D");
    map.insert(KeyboardKey::ArrowRight, "\x1b[C");
    map.insert(KeyboardKey::ArrowUp, "\x1b[A");
    map.insert(KeyboardKey::ArrowDown, "\x1b[B");
    map.insert(KeyboardKey::Home, "\x1bOH");
    map.insert(KeyboardKey::PageUp, "\x1b[5~");
    map.insert(KeyboardKey::PageDown, "\x1b[6~");
    map.insert(KeyboardKey::F1, "\x1bOP");
    map.insert(KeyboardKey::F2, "\x1bOQ");
    map.insert(KeyboardKey::F3, "\x1bOR");
    map.insert(KeyboardKey::F4, "\x1bOS");
    map.insert(KeyboardKey::F5, "\x1b[15~");
    map.insert(KeyboardKey::F6, "\x1b[17~");
    map.insert(KeyboardKey::F7, "\x1b[18~");
    map.insert(KeyboardKey::F8, "\x1b[19~");
    map.insert(KeyboardKey::F9, "\x1b[20~");
    map.insert(KeyboardKey::F10, "\x1b[21~");
    map.insert(KeyboardKey::F11, "\x1b[23~");
    map.insert(KeyboardKey::F12, "\x1b[24~");
    map.insert(KeyboardKey::Number0, "0");
    map.insert(KeyboardKey::Number1, "1");
    map.insert(KeyboardKey::Number2, "2");
    map.insert(KeyboardKey::Number3, "3");
    map.insert(KeyboardKey::Number4, "4");
    map.insert(KeyboardKey::Number5, "5");
    map.insert(KeyboardKey::Number6, "6");
    map.insert(KeyboardKey::Number7, "7");
    map.insert(KeyboardKey::Number8, "8");
    map.insert(KeyboardKey::Number9, "9");
    map.insert(KeyboardKey::Add, "+");
    map.insert(KeyboardKey::Multiply, "*");
    map.insert(KeyboardKey::Subtract, "-");
    map.insert(KeyboardKey::Grave, "`");
    map.insert(KeyboardKey::LeftBrace, "[");
    map.insert(KeyboardKey::RightBrace, "]");
    map.insert(KeyboardKey::SemiColon, ";");
    map.insert(KeyboardKey::Apostrophe, "'");
    map.insert(KeyboardKey::BackwardSlash, "\\");
    map.insert(KeyboardKey::Comma, ",");
    map.insert(KeyboardKey::Period, ".");
    map.insert(KeyboardKey::Slash, "/");
    map.insert(KeyboardKey::CapsLock, "\x1b[?25h");
    map.insert(KeyboardKey::ScrollLock, "\x1b[?45h");
    map.insert(KeyboardKey::NumLock, "\x1b[?69h");
    map.insert(KeyboardKey::PrintScreen, "\x1b[?47h");
    map.insert(KeyboardKey::Insert, "\x1b[2~");
    map
}

pub fn start_rtinterpreter() -> (mpsc::Sender<KeyboardKey>, mpsc::Sender<bool>) {
    let (keysender, keyreceiver) = mpsc::channel();
    let (wordsender, wordreceiver) = mpsc::channel();
    let (shutdown, gsd_receiver) = mpsc::channel();
    let (buf1sd_sender, buf1sd_receiver) = mpsc::channel();
    let (buf2sd_sender, buf2sd_receiver) = mpsc::channel();
    let keymap = init_keymapper(); // Modified keymapper function
    let buffer1 = Arc::new(Mutex::new(Vec::new()));
    let buffer2 = Arc::new(Mutex::new(Vec::new()));

    // Clone Arcs for threads
    let buffer1_clone = Arc::clone(&buffer1);
    let buffer2_clone = Arc::clone(&buffer2);

    //Spawning buffer1 and buffer2 threads
    let _buffer1_thread = thread::spawn(move || {
        init_buffer1(keyreceiver, wordsender, buf1sd_receiver, buffer1_clone, &keymap);
    });
    let _buffer2_thread = thread::spawn(move || {
        init_buffer2(wordreceiver, buf2sd_receiver, buffer2_clone);
    });
    //Spawn the Shutdown Thread
    let _shutdown_thread = thread::spawn(move || {
        rtinterpreter_shutdown(gsd_receiver, buf1sd_sender, buf2sd_sender);
    });

    (keysender, shutdown)
}

fn init_buffer1(
    keyreceiver: mpsc::Receiver<KeyboardKey>,
    wordsender: mpsc::Sender<String>,
    shutdown: mpsc::Receiver<bool>,
    buffer1: Arc<Mutex<Vec<String>>>,
    keymap: &HashMap<KeyboardKey, &'static str>,
) {
    let mut buffer1_guard = buffer1.lock().unwrap(); // Acquire lock once

    loop {
        match keyreceiver.try_recv() {
            Ok(key) => {
                match key {
                    KeyboardKey::Space | KeyboardKey::Period | KeyboardKey::Comma | KeyboardKey::Enter=> {
                        if !buffer1_guard.is_empty() {
                            if let Some(&key_str) = keymap.get(&key) {
                                if let Err(e) = execute!(std::io::stdout(), Print(key_str)) {
                                    eprintln!("Failed to print: {}", e);
                                }
                            }
                            wordsender.send(buffer1_guard.drain(..).collect()).unwrap();
                        }
                    }
                    _ => {
                        if let Some(&key_str) = keymap.get(&key) {
                            if let Err(e) = execute!(std::io::stdout(), Print(key_str)) {
                                eprintln!("Failed to print: {}", e);
                            }
                            buffer1_guard.push(key_str.to_string());
                        }
                    }
                }
            }
            Err(_) => continue,
        }

        if shutdown.try_recv() == Ok(true) {
            println!("Buffer1 shutdown signal received");
            break;
        }
    }
}



fn init_buffer2(
    wordreceiver: mpsc::Receiver<String>,
    shutdown: mpsc::Receiver<bool>,
    buffer2: Arc<Mutex<Vec<String>>>,
) {
    let rwords = rwords_gen();
    loop {
        match wordreceiver.recv() {
            Ok(word) => {
                let mut buffer2_guard = buffer2.lock().unwrap();
                buffer2_guard.push(word.to_string());

                let mut violation: bool = false;

                for rword in rwords.iter().filter(|&rword| word.contains(rword)) {
                    violation = true;
                    if let Err(_e) = execute!(
                        std::io::stdout(),
                        SetForegroundColor(Color::Red),
                        Print(format!("\nRestricted word found: {}", rword)),
                        ResetColor,
                    ) {
                        continue;
                    }
                    println!();
                }

                if violation {
                    super::audiocontrol::init_censor();
                }
            }
            Err(_) => println!("Error receiving word"), // Channel disconnected, exit loop
        }

        match shutdown.try_recv() {
            Ok(true) => {
                break;
            }
            Ok(false) | Err(_) => continue,
        }
    }
}


fn rwords_gen() -> HashSet<&'static str>{
    let mut words = HashSet::new();

    // Using the tinyfiledialogs library to select a text file
    let dir = 
    match open_file_dialog("Select the text file containing restricted keywords", "", Some((&["*.txt"], "Text files"))) {
        Some(file_path) => file_path,
        _ => {
            println!("No list of words provided, proceeding with an empty list.");
            "".to_string()
        }
    };
    if dir != "" {
        let rwords_file = File::open(dir).unwrap();

        // Populate HashSet with words from text file using a BufferedReader
        for line in BufReader::new(rwords_file).lines() {
            if let Ok(word) = line {
                // Convert the String into a &'static str and insert into the HashSet
                words.insert(Box::leak(word.into_boxed_str()) as &'static str);
            } else {
                continue
            }
        }
    }

    // Return HashSet
    words
}

fn rtinterpreter_shutdown(gsd: mpsc::Receiver<bool>, buf1sd: mpsc::Sender<bool>, buf2sd: mpsc::Sender<bool>) {
    loop {
        match gsd.try_recv() {
            Ok(true) => {
                // Shutdown signal received, send shutdown signals to both buffers
                let _ = buf1sd.send(true);
                let _ = buf2sd.send(true);
                break;
            }
            Err(TryRecvError::Empty) => {
                // No message available, sleep for a short duration and try again
                std::thread::sleep(Duration::from_millis(10));
            }
            Ok(false) | Err(_) => {
                // Other errors or false received, continue the loop
                continue;
            }
        }
    }
}
