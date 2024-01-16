//! Getting multiple cameras to work right is easier if all config lives
//! together like a happy family.

/// One pixel is 3x3 pixels on screen.
pub const PIXEL_ZOOM: i32 = 3;

/// What's shown on screen with [`PIXEL_ZOOM`].
pub const PIXEL_VISIBLE_WIDTH: f32 = 640.0;
/// What's shown on screen [`PIXEL_ZOOM`].
pub const PIXEL_VISIBLE_HEIGHT: f32 = 360.0;

pub mod render_layer {
    //! Render layers are assigned to entities and cameras to decide what is
    //! rendered where.

    /// Objects and characters.
    pub const OBJ: u8 = 1;
    /// Background images.
    pub const BG: u8 = 2;
    /// Dialog entities such as portrait, text box and all.
    pub const DIALOG: u8 = 25;
    /// Light scene
    pub const LIGHT: u8 = 29;
    /// Loading screen entities
    pub const LOADING: u8 = 21;
}

pub mod order {
    //! The higher the order, the later the camera is rendered into the
    //! viewport.

    /// The main camera in each scene
    pub const DEFAULT: isize = 1;
    /// The camera that renders the light scene is above the main camera to
    /// illuminate the scene.
    pub const LIGHT: isize = 2;
    /// Dialog is overlaid on top of everything else.
    pub const DIALOG: isize = 11;
    /// The camera that renders the loading screen is above the main camera
    /// because we smoothly change opacity from 0 to 1 and back.
    pub const LOADING: isize = 12;
    /// Overlay for devtools.
    pub const DEV: isize = 420;
}
