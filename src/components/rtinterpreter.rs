use std::{
    collections::{HashMap, HashSet}, fs::{self, File, OpenOptions}, io::{Read, Write, BufRead, BufReader}, sync::{mpsc::{self, TryRecvError},Arc, Mutex}, thread,
    time::Duration,
};
use lettre::{
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
    message::{Attachment, MessageBuilder, MultiPart, SinglePart}
};
use tinyfiledialogs::open_file_dialog;
use willhook::KeyboardKey;
use crossterm::{execute, style::{Print, SetForegroundColor, Color, ResetColor}};
use tts_rust::{ tts::GTTSClient, languages::Languages };
use dotenv::dotenv;

use super::filecreator;

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
    map.insert(KeyboardKey::BackSpace, " BackSpace ");
    map.insert(KeyboardKey::Tab, "\t");
    map.insert(KeyboardKey::Escape, " Esc ");
    map.insert(KeyboardKey::Delete, " Del ");
    map.insert(KeyboardKey::ArrowLeft, " Left ");
    map.insert(KeyboardKey::ArrowRight, " Right ");
    map.insert(KeyboardKey::ArrowUp, " Up ");
    map.insert(KeyboardKey::ArrowDown, " Down ");
    map.insert(KeyboardKey::Home, " Home ");
    map.insert(KeyboardKey::PageUp, " PgUp ");
    map.insert(KeyboardKey::PageDown, " PgDown ");
    map.insert(KeyboardKey::F1, "F1");
    map.insert(KeyboardKey::F2, "F2");
    map.insert(KeyboardKey::F3, "F3");
    map.insert(KeyboardKey::F4, "F4");
    map.insert(KeyboardKey::F5, "F5");
    map.insert(KeyboardKey::F6, "F6");
    map.insert(KeyboardKey::F7, "F7");
    map.insert(KeyboardKey::F8, "F8");
    map.insert(KeyboardKey::F9, "F9");
    map.insert(KeyboardKey::F10, "F10");
    map.insert(KeyboardKey::F11, "F11");
    map.insert(KeyboardKey::F12, "F12");
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
    map.insert(KeyboardKey::Numpad0, "0");
    map.insert(KeyboardKey::Numpad1, "1");
    map.insert(KeyboardKey::Numpad2, "2");
    map.insert(KeyboardKey::Numpad3, "3");
    map.insert(KeyboardKey::Numpad4, "4");
    map.insert(KeyboardKey::Numpad5, "5");
    map.insert(KeyboardKey::Numpad6, "6");
    map.insert(KeyboardKey::Numpad7, "7");
    map.insert(KeyboardKey::Numpad8, "8");
    map.insert(KeyboardKey::Numpad9, "9");
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
    map.insert(KeyboardKey::CapsLock, " CapsLock ");
    map.insert(KeyboardKey::ScrollLock, " ScrollLock ");
    map.insert(KeyboardKey::NumLock, " NumLock ");
    map.insert(KeyboardKey::PrintScreen, " PrintScreen ");
    map.insert(KeyboardKey::Insert, " Insert ");
    map.insert(KeyboardKey::LeftControl, " Ctrl ");
    map.insert(KeyboardKey::RightControl, " Ctrl ");
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
    let path = filecreator::retrieve_idirectory().unwrap();
    if !path.exists() {
        if let Err(e) = fs::create_dir_all(&path) {
            eprintln!("Error creating directory: {}", e);
            panic!("{}",e);
        }
    }
    let mut ifile = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true) // Truncate existing content
        .open(path.join("rti_out.txt")).unwrap();
    loop {
        match keyreceiver.try_recv() {
            Ok(key) => {
                match key {
                    KeyboardKey::Space | KeyboardKey::Period | KeyboardKey::Comma | KeyboardKey::Enter=> {
                        if !buffer1_guard.is_empty() {
                            if let Some(&key_str) = keymap.get(&key) {
                                // if let Err(e) = execute!(std::io::stdout(), Print(key_str)) {
                                //     eprintln!("Failed to print: {}", e);
                                // }
                                let _ = &ifile.write(key_str.as_bytes()).unwrap();
                            }
                            wordsender.send(buffer1_guard.drain(..).collect()).unwrap();
                        }
                    }
                    _ => {
                        if let Some(&key_str) = keymap.get(&key) {
                            // if let Err(e) = execute!(std::io::stdout(), Print(key_str)) {
                            //     eprintln!("Failed to print: {}", e);
                            // }
                            let _ = &ifile.write(key_str.as_bytes()).unwrap();
                            buffer1_guard.push(key_str.to_string());
                        }
                    }
                }
            }
            Err(_) => continue,
        }

        if shutdown.try_recv() == Ok(true) {
            drop(ifile);
            break;
        }
    }
}



fn init_buffer2(
    wordreceiver: mpsc::Receiver<String>,
    shutdown: mpsc::Receiver<bool>,
    buffer2: Arc<Mutex<Vec<String>>>,
) {
    let mut violations = 0;
    let mut email_sent = false;
    let narrator: GTTSClient = GTTSClient {
        volume: 1.0, 
        language: Languages::English, // use the Languages enum
        tld: "com",
    };
    let rwords = rwords_gen();
    let path = filecreator::retrieve_rdirectory().unwrap();
    if !path.exists() {
        if let Err(e) = fs::create_dir_all(&path) {
            eprintln!("Error creating directory: {}", e);
            panic!("{}",e);
        }
    }
    let mut rfile = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true) // Truncate existing content
        .open(path.join("violations.txt")).unwrap();
    loop {
        match wordreceiver.recv() {
            Ok(word) => {
                let mut buffer2_guard = buffer2.lock().unwrap();
                buffer2_guard.push(word.to_string());

                let mut violation: bool = false;

                for rword in rwords.iter().filter(|&rword| word.contains(rword)) {
                    violation = true;
                    violations += 1;
                    if let Err(_e) = execute!(
                        std::io::stdout(),
                        SetForegroundColor(Color::Red),
                        Print(format!("\nRestricted word found: {}", rword)),
                        ResetColor,
                    ) {
                        continue;
                    }
                    println!();
                    let _ = &rfile.write(format!("{}\n", rword).as_bytes()).unwrap();
                }

                if violation {
                    if violations >= 3{
                        let _ = narrator.speak("Violations exceeded 3, Administrator has been notifed of the violations in this session");
                        if !email_sent {
                            send_email();
                        }
                    } else {
                        super::audiocontrol::init_censor();
                    }
                }
            }
            Err(_) => println!("Error receiving word"), // Channel disconnected, exit loop
        }

        if shutdown.try_recv() == Ok(true) {
            drop(rfile);
            break;
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


fn send_email(){
    dotenv().ok();
    let smtp_key: &str = &std::env::var("SMTP_KEY").expect("SMTP_KEY not set");
    let from_email: &str = &std::env::var("SENDER").unwrap();
    let to_email: &str = &std::env::var("TO").unwrap();
    let host: &str = &std::env::var("HOST").unwrap();

    let ifile_path = filecreator::retrieve_idirectory().unwrap().join("rti_out.txt");
    let mut ifile = File::open(&ifile_path).expect("Failed to open interpreter output file");
    let mut ifile_data = String::new();
    ifile.read_to_string(&mut ifile_data).expect("Failed to read interpreter output file");

    let rfile_path = filecreator::retrieve_rdirectory().unwrap().join("violations.txt");
    let mut rfile = File::open(&rfile_path).expect("Failed to open interpreter output file");
    let mut rfile_data = String::new();
    rfile.read_to_string(&mut rfile_data).expect("Failed to read interpreter output file");

    let ifile_size = fs::metadata(&ifile_path)
    .map(|metadata| metadata.len() < 25 * 1024 * 1024)
    .unwrap_or(false);
    // println!("ifile status: {}", ifile_size);

    let rfile_size = fs::metadata(&rfile_path)
        .map(|metadata| metadata.len() < 25 * 1024 * 1024)
        .unwrap_or(false);
    // println!("rfile status: {}", rfile_size);
    let emailbuild: MessageBuilder = Message::builder()
        .from(from_email.parse().unwrap())
        .to(to_email.parse().unwrap())
        .subject("Spektra Violation Report");

    let email: Message;

    if ifile_size && rfile_size {
        email = emailbuild.multipart(MultiPart::mixed()
        .singlepart(SinglePart::plain(String::from(format!("There has been several violations in Computer {} of Lab {}\n", &std::env::var("COMPUTER").unwrap(), &std::env::var("LABNAME").unwrap()))))
        .singlepart(SinglePart::plain(String::from("The outputs have been attached")))
        .singlepart(Attachment::new("RTI.txt".parse().unwrap()).body(ifile_data, "text/plain".parse().unwrap()))
        .singlepart(Attachment::new("Violations.txt".parse().unwrap()).body(rfile_data, "text/plain".parse().unwrap())))
        .unwrap();
        let creds: Credentials = Credentials::new(from_email.to_string(), smtp_key.to_string());

        // Open a remote connection to gmail
        let mailer: SmtpTransport = SmtpTransport::relay(&host)
            .unwrap()
            .credentials(creds)
            .build();

        // Send the email
        match mailer.send(&email) {
            Ok(_) => println!("Email sent successfully!"),
            Err(e) => panic!("Could not send email: {:?}", e),
        }
    }
}