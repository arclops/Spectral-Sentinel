use winapi::{
    um::{
        endpointvolume::IAudioEndpointVolume,
        mmdeviceapi::{eRender, IMMDevice, IMMDeviceEnumerator, CLSID_MMDeviceEnumerator},
        combaseapi::{CoInitializeEx, CoUninitialize, CLSCTX_ALL},
    },
    shared::winerror::S_OK,
    Interface,
};
use std::{
    sync::mpsc, thread, time::Duration
};
use rodio::{
    OutputStream, Sink,
};
use tts_rust::{ tts::GTTSClient, languages::Languages };

pub fn init_censor(rword_orig: &str, mode: i32) {
    let (audio_control_sender, audio_control_receiver) = mpsc::channel();
    let (audio_stop_sender, audio_stop_receiver) = mpsc::channel();
    let narrator: GTTSClient = GTTSClient {
        volume: 1.0, 
        language: Languages::English, // use the Languages enum
        tld: "com",
    };
    // Spawn thread for audio manipulation
    let audio_control_thread = thread::spawn(move || {
        manipulate_audio(audio_control_receiver, audio_stop_receiver);
    });
    let rword= String::from(rword_orig);
    
    // Spawn thread for playing sound
    let audio_play_thread = thread::spawn(move || {
        play_sound(audio_control_sender, rword, &narrator, mode);
    });

    // Wait for the audio playback to complete
    audio_play_thread.join().unwrap();

    // Send stop signal to the audio manipulation thread
    audio_stop_sender.send(()).unwrap();

    // Wait for the audio manipulation thread to terminate
    audio_control_thread.join().unwrap();
}

fn manipulate_audio(_control_receiver: mpsc::Receiver<()>, stop_receiver: mpsc::Receiver<()>) {
    unsafe {
        // Initialize COM library
        let hr = CoInitializeEx(std::ptr::null_mut(), 0x2); // COINIT_APARTMENTTHREADED = 0x2
        if hr != S_OK {
            panic!("Failed to initialize COM library");
        }

        // Create device enumerator
        let mut enumerator: *mut IMMDeviceEnumerator = std::ptr::null_mut();
        let hr = winapi::um::combaseapi::CoCreateInstance(
            &CLSID_MMDeviceEnumerator,
            std::ptr::null_mut(),
            CLSCTX_ALL,
            &IMMDeviceEnumerator::uuidof(),
            &mut enumerator as *mut _ as *mut _,
        );
        if hr != S_OK {
            panic!("Failed to create device enumerator");
        }

        // Get default audio endpoint
        let mut endpoint: *mut IMMDevice = std::ptr::null_mut();
        let hr = (*enumerator).GetDefaultAudioEndpoint(0, eRender, &mut endpoint);
        if hr != S_OK {
            panic!("Failed to get default audio endpoint");
        }

        // Activate audio endpoint volume interface
        let mut volume: *mut IAudioEndpointVolume = std::ptr::null_mut();
        let hr = (*endpoint).Activate(
            &IAudioEndpointVolume::uuidof(),
            CLSCTX_ALL,
            std::ptr::null_mut(),
            &mut volume as *mut _ as *mut _,
        );
        if hr != S_OK {
            panic!("Failed to activate audio endpoint volume");
        }

        let mut is_playing = true;

        // Main loop to continuously monitor audio status
        while is_playing {
            // Check if currently muted
            let mut is_muted: i32 = 0;
            let hr = (*volume).GetMute(&mut is_muted);
            if hr != S_OK {
                panic!("Failed to get mute status");
            }

            // If muted, unmute and set volume to 100%
            if is_muted != 0 {
                let hr = (*volume).SetMute(0, std::ptr::null());
                if hr != S_OK {
                    panic!("Failed to unmute audio");
                }
            }
            (*volume).SetMasterVolumeLevelScalar(1.0, std::ptr::null());

            // Check if stop signal is received
            if let Ok(_) = stop_receiver.try_recv() {
                is_playing = false;
            }

            // Sleep for a short duration before checking again
            thread::sleep(Duration::from_millis(100));
        }

        // Cleanup
        (*volume).Release();
        (*endpoint).Release();
        (*enumerator).Release();
        CoUninitialize();
    }
}

fn play_sound(control_sender: mpsc::Sender<()>, rword: String, narrator: &GTTSClient, mode: i32) {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    
    // Add a dummy source for the sake of the example.
    if mode != 0 {
        gen_melody(narrator, rword);
    } else {
        gen_voice(narrator, rword);
    }
    
    // The sound plays in a separate thread. This call will block the current thread until the sink
    // has finished playing all its queued sounds.
    sink.sleep_until_end();

    // Signal audio control thread that playback has finished
    control_sender.send(()).unwrap();
}

fn gen_melody(narrator: &GTTSClient, rword: String) {
    // let mut source = SineWave::new(440.0).take_duration(Duration::from_secs_f32(1.0)).amplify(30.0);
    // sink.append(source);
    // let frequencies = [261.3, 293.66, 329.63, 261.63, 329.63, 349.23, 329.63, 293.66, 261.63];
    // for &freq in frequencies.iter() {
    //     source = SineWave::new(freq)
    //         .take_duration(Duration::from_secs_f32(0.5))
    //         .amplify(30.0);
    //     sink.append(source);
    // }
    let _ = narrator.speak(format!("Restricted keyword found: {:?}",rword).as_str());
}

fn gen_voice(narrator: &GTTSClient, line: String) {
    let _ = narrator.speak(line.as_str());
}

//let _ = narrator.speak(format!("Restricted keyword found: {:?}",rword).as_str());