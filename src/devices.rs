use rocket::serde::Serialize;
#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CaptureConfig {
    #[serde(rename = "type")]
    device_type: String,
    channels: usize,
    filename: String,
    format: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct PlaybackConfig {
    #[serde(rename = "type")]
    device_type: String,
    channels: usize,
    device: String,
    format: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Devices {
    samplerate: String, //using ALSA plugin
    chunksize: i32,
    queuelimit: i32,
    capture: CaptureConfig,
    playback: PlaybackConfig,
}

//in the future, select a device
impl Devices {
    // will input channels be consistent??  will I always get 8 channels of PCM over HDMI even if its stereo?
    pub fn init(input_channels: usize, output_channels: usize) -> Self {
        Self {
            samplerate: "$samplerate$".to_string(),
            chunksize: 1024,
            queuelimit: 1,
            capture: CaptureConfig {
                device_type: "File".to_string(),
                channels: input_channels,
                filename: "/dev/stdin".to_string(),
                format: "$format$".to_string(),
            },
            playback: PlaybackConfig {
                device_type: "Alsa".to_string(),
                channels: output_channels,
                device: "hw:1,0".to_string(),
                format: "S16LE".to_string(),
            },
        }
    }
    pub fn okto_dac8(input_channels: usize, output_channels: usize) -> Self {
        Self {
            samplerate: "$samplerate$".to_string(),
            chunksize: 1024,
            queuelimit: 1,
            capture: CaptureConfig {
                device_type: "File".to_string(),
                channels: input_channels,
                filename: "/dev/stdin".to_string(),
                format: "$format$".to_string(),
            },
            playback: PlaybackConfig {
                device_type: "Alsa".to_string(),
                channels: output_channels,
                device: "hw:DAC8PRO".to_string(),
                format: "S32LE".to_string(),
            },
        }
    }

    pub fn topping_dm7(input_channels: usize, output_channels: usize) -> Self {
        Self {
            samplerate: "$samplerate$".to_string(),
            chunksize: 1024,
            queuelimit: 1,
            capture: CaptureConfig {
                device_type: "File".to_string(),
                channels: input_channels,
                filename: "/dev/stdin".to_string(),
                format: "$format$".to_string(),
            },
            playback: PlaybackConfig {
                device_type: "Alsa".to_string(),
                channels: output_channels,
                device: "hw:DM7".to_string(),
                format: "S32LE".to_string(),
            },
        }
    }
    pub fn motu_mk5(input_channels: usize, output_channels: usize) -> Self {
        Self {
            samplerate: "$samplerate$".to_string(),
            chunksize: 1024,
            queuelimit: 1,
            capture: CaptureConfig {
                device_type: "File".to_string(),
                channels: input_channels,
                filename: "/dev/stdin".to_string(),
                format: "$format$".to_string(),
            },
            playback: PlaybackConfig {
                device_type: "Alsa".to_string(),
                channels: output_channels,
                device: "hw:UltraLitemk5".to_string(),
                format: "S24LE3".to_string(),
            },
        }
    }
}