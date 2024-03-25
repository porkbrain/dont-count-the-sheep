//! Various frontends for dialog.

pub mod portrait;

/// Different kids of FE that render dialog
pub enum DialogFrontend {
    /// Includes the character portrait, classic dialog box, with options.
    Portrait,
}
