use crate::utils::image_utils::RgbWrapper;

pub const PPM_FORMAT_VERSION: u16 = 0x24;
pub const PPM_THUMBNAIL_SIZE: usize = 1536;
pub const PPM_THUMBNAIL_COLORS: [&str; 16] = [
    "#FFFFFF", "#525252", "#FFFFFF", "#9C9C9C", "#FF4844", "#C8514F", "#FFADAC", "#00FF00",
    "#4840FF", "#514FB8", "#ADABFF", "#00FF00", "#B657B7", "#00FF00", "#00FF00", "#00FF00",
];

pub const PPM_PAPER_COLORS: [RgbWrapper; 2] = [
    RgbWrapper {
        r: 255,
        g: 255,
        b: 255,
    },
    RgbWrapper {
        r: 14,
        g: 14,
        b: 14,
    },
];
pub const PPM_COLOR_RED: RgbWrapper = RgbWrapper {
    r: 255,
    g: 42,
    b: 42,
};
pub const PPM_COLOR_BLUE: RgbWrapper = RgbWrapper {
    r: 10,
    g: 57,
    b: 255,
};

pub const PPM_FRAMERATE: [f32; 9] = [0.5, 0.5, 1.0, 2.0, 4.0, 6.0, 12.0, 20.0, 30.0];

pub const ADPCM_STATE_HEADER_SIZE: usize = 4;

// Flipnote Studio public key, used to verify the signature of a PPM file.
// This can NOT be used to sign them.
pub const FLIPNOTE_STUDIO_PUBLIC_KEY: &str = "-----BEGIN PUBLIC KEY-----
MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQDCPLwTL6oSflv+gjywi/sM0TUB
90xqOvuCpjduETjPoN2FwMebxNjdKIqHUyDu4AvrQ6BDJc6gKUbZ1E27BGZoCPH4
9zQRb+zAM6M9EjHwQ6BABr0u2TcF7xGg2uQ9MBWz9AfbVQ91NjfrNWo0f7UPmffv
1VvixmTk1BCtavZxBwIDAQAB
-----END PUBLIC KEY-----";

pub const PPM_AUDIO_SAMPLE_RATE: i32 = 8180;
pub const PPM_AUDIO_PLAYBACK_SAMPLE_RATE: i32 = 32720;

//offsets we need
pub const PPM_OFFSET_AUDIO_DATA_SIZE: u64 = 0x8;
