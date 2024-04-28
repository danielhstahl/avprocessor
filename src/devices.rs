use rocket::serde::Serialize;
#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CaptureConfig {
    #[serde(rename = "type")]
    device_type: String,
    channels: usize,
    device: String,
    format: Option<String>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct PlaybackConfig {
    #[serde(rename = "type")]
    device_type: String,
    channels: usize,
    device: String,
    format: Option<String>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Devices {
    samplerate: i32, //using ALSA plugin
    chunksize: i32,
    queuelimit: i32,
    capture: CaptureConfig,
    playback: PlaybackConfig,
}

//in the future, select a device
impl Devices {
    // will input channels be consistent??  will I always get 8 channels of PCM over HDMI even if its stereo?
    pub fn okto_dac8(input_channels: usize, output_channels: usize) -> Self {
        Self {
            samplerate: 96000, //high sample rate; should be transparent
            chunksize: 2048,
            queuelimit: 1,
            capture: CaptureConfig {
                device_type: "Alsa".to_string(),
                channels: input_channels,
                device: "hw:Loopback,1".to_string(),
                format: Some("S32LE".to_string()),
            },
            playback: PlaybackConfig {
                device_type: "Alsa".to_string(),
                channels: output_channels,
                device: "hw:DAC8PRO".to_string(),
                format: Some("S32LE".to_string()),
            },
        }
    }

    pub fn hdmi_osmc_pi(input_channels: usize, output_channels: usize) -> Self {
        Self {
            samplerate: 96000, //high sample rate; should be transparent
            chunksize: 2048,
            queuelimit: 1,
            capture: CaptureConfig {
                device_type: "Alsa".to_string(),
                channels: input_channels,
                device: "hw:Loopback,1".to_string(),
                format: Some("S24LE3".to_string()),
            },
            playback: PlaybackConfig {
                device_type: "Alsa".to_string(),
                channels: output_channels,
                device: "hw:vc4hdmi".to_string(),
                format: Some("S24LE3".to_string()),
            },
        }
    }

    pub fn topping_dm7(input_channels: usize, output_channels: usize) -> Self {
        Self {
            samplerate: 96000, //high sample rate; should be transparent
            chunksize: 2048,
            queuelimit: 1,
            capture: CaptureConfig {
                device_type: "Alsa".to_string(),
                channels: input_channels,
                device: "hw:Loopback,1".to_string(),
                format: Some("S32LE".to_string()),
            },
            playback: PlaybackConfig {
                device_type: "Alsa".to_string(),
                channels: output_channels,
                device: "hw:DM7".to_string(),
                format: Some("S32LE".to_string()),
            },
        }
    }
    pub fn motu_mk5(input_channels: usize, output_channels: usize) -> Self {
        Self {
            samplerate: 96000, //high sample rate; should be transparent
            chunksize: 2048,
            queuelimit: 1,
            capture: CaptureConfig {
                device_type: "Alsa".to_string(),
                channels: input_channels,
                device: "hw:Loopback,1".to_string(),
                format: Some("S24LE3".to_string()),
            },
            playback: PlaybackConfig {
                device_type: "Alsa".to_string(),
                channels: output_channels,
                device: "hw:UltraLitemk5".to_string(),
                format: Some("S24LE3".to_string()),
            },
        }
    }
}
