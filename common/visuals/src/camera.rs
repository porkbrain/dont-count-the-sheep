/// One pixel is 3x3 pixels on screen.
pub const PIXEL_ZOOM: i32 = 3;

/// What's shown on screen with [`PIXEL_ZOOM`].
pub const PIXEL_VISIBLE_WIDTH: f32 = 640.0;
/// What's shown on screen [`PIXEL_ZOOM`].
pub const PIXEL_VISIBLE_HEIGHT: f32 = 360.0;

pub mod render_layer {
    pub const BG: u8 = 2;
    pub const OBJ: u8 = 1;
    pub const DIALOG: u8 = 25;
    pub const LIGHT: u8 = 29;
    pub const LOADING: u8 = 21;
}

pub mod order {
    pub const DEFAULT: isize = 1;
    pub const LIGHT: isize = 2;
    pub const LOADING: isize = 10;
    pub const DIALOG: isize = 12;
}
