use bevy::prelude::*;
use crossbeam_channel::Receiver;

#[derive(Bundle, Default)]
pub struct WebpBundle {
    pub frame_rate: FrameRate,
    pub animation: Handle<WebpAnimation>,
    pub sprite: Sprite,
    pub target: Handle<Image>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

#[derive(Component)]
pub struct FrameRate {
    /// How many ticks on fixed schedule to wait before advancing to the next
    /// frame.
    pub ticks_per_frame: u32,
    pub current_tick: u32,
}

#[derive(Asset, TypePath, Debug, Clone)]
pub struct WebpAnimation {
    pub(crate) next_frame: Receiver<Image>,
    pub(crate) label: String,
}

impl Default for FrameRate {
    fn default() -> Self {
        Self {
            ticks_per_frame: 1,
            current_tick: 0,
        }
    }
}

impl FrameRate {
    pub fn new(ticks_per_frame: u32) -> Self {
        Self {
            ticks_per_frame,
            current_tick: 0,
        }
    }
}
