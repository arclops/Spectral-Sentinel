use winapi::{
    um::{
        endpointvolume::IAudioEndpointVolume,
        mmdeviceapi::{eRender, IMMDevice, IMMDeviceEnumerator,CLSID_MMDeviceEnumerator},
        combaseapi::{CoInitializeEx, CoUninitialize, CLSCTX_ALL},
    },
    shared::winerror::S_OK,
    Interface,
};
use std::time::Duration;
use rodio::{
    OutputStream, Sink,
    source::{SineWave, Source}
};

pub fn init_censor() {
    enable_master_audio();
    start_beep();
}

fn enable_master_audio() {
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

        // Check if currently muted
        let mut is_muted: i32 = 0;
        let hr = (*volume).GetMute(&mut is_muted);
        if hr != S_OK {
            panic!("Failed to get mute status");
        }

        // Toggle mute status
        if is_muted != 0 {
            // If currently muted, unmute
            let hr = (*volume).SetMute(0, std::ptr::null()); // 0 represents false (unmuted)
            if hr != S_OK {
                panic!("Failed to unmute audio");
            }
        }

        // Set master volume (0.0 to 1.0)
        let volume_level = 1.0; // Example: set volume to 50%
        (*volume).SetMasterVolumeLevelScalar(volume_level as f32, std::ptr::null());

        // Cleanup
        (*volume).Release();
        (*endpoint).Release();
        (*enumerator).Release();
        CoUninitialize();
    }
}


fn start_beep() {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    
    // Add a dummy source of the sake of the example.
    let source = SineWave::new(440.0).take_duration(Duration::from_secs_f32(30.0)).amplify(30.0);
    sink.append(source);
    
    // The sound plays in a separate thread. This call will block the current thread until the sink
    // has finished playing all its queued sounds.
    sink.sleep_until_end();
    }