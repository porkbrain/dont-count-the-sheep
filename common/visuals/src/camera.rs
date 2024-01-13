//! Getting multiple cameras to work right is easier if all config lives
//! together like a happy family.

#![allow(missing_docs)]

/// One pixel is 3x3 pixels on screen.
pub const PIXEL_ZOOM: i32 = 3;

/// What's shown on screen with [`PIXEL_ZOOM`].
pub const PIXEL_VISIBLE_WIDTH: f32 = 640.0;
/// What's shown on screen [`PIXEL_ZOOM`].
pub const PIXEL_VISIBLE_HEIGHT: f32 = 360.0;

pub mod render_layer {
    //! Render layers are assigned to entities and cameras to decide what is
    //! rendered where.

    pub const BG: u8 = 2;
    pub const OBJ: u8 = 1;
    pub const DIALOG: u8 = 25;
    pub const LIGHT: u8 = 29;
    pub const LOADING: u8 = 21;
}

pub mod order {
    //! The higher the order, the later the camera is rendered into the
    //! viewport.

    pub const DEFAULT: isize = 1;
    pub const LIGHT: isize = 2;
    pub const LOADING: isize = 10;
    pub const DIALOG: isize = 12;
}
