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

pub const ADPCM_STEP_TABLE: [i32; 90] = [
    7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 19, 21, 23, 25, 28, 31, 34, 37, 41, 45, 50, 55, 60, 66,
    73, 80, 88, 97, 107, 118, 130, 143, 157, 173, 190, 209, 230, 253, 279, 307, 337, 371, 408, 449,
    494, 544, 598, 658, 724, 796, 876, 963, 1060, 1166, 1282, 1411, 1552, 1707, 1878, 2066, 2272,
    2499, 2749, 3024, 3327, 3660, 4026, 4428, 4871, 5358, 5894, 6484, 7132, 7845, 8630, 9493,
    10442, 11487, 12635, 13899, 15289, 16818, 18500, 20350, 22385, 24623, 27086, 29794, 32767, 0,
];

pub const ADPCM_INDEX_TABLE: [i32; 16] = [-1, -1, -1, -1, 2, 4, 6, 8, -1, -1, -1, -1, 2, 4, 6, 8];

// Flipnote Studio public key, used to verify the signature of a PPM file.
// This can NOT be used to sign them.
pub const FLIPNOTE_STUDIO_PUBLIC_KEY: &str = "-----BEGIN PUBLIC KEY-----
MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQDCPLwTL6oSflv+gjywi/sM0TUB
90xqOvuCpjduETjPoN2FwMebxNjdKIqHUyDu4AvrQ6BDJc6gKUbZ1E27BGZoCPH4
9zQRb+zAM6M9EjHwQ6BABr0u2TcF7xGg2uQ9MBWz9AfbVQ91NjfrNWo0f7UPmffv
1VvixmTk1BCtavZxBwIDAQAB
-----END PUBLIC KEY-----";

pub const PPM_AUDIO_SAMPLE_RATE: i32 = 8180;