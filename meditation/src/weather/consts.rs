use crate::prelude::*;

pub(crate) const DEFAULT_TRANSFORM: Transform = Transform {
    translation: Vec3::new(0.0, 0.0, zindex::WEATHER),
    rotation: Quat::from_array([0.0, 0.0, 0.0, 1.0]),
    scale: Vec3::new(1.0, 1.0, 1.0),
};

/// Cannot jump more times in a row than this before resetting
pub(crate) const MAX_JUMPS: usize = 4;

/// How fast does weather rotate towards its velocity vector
pub(crate) const ROTATION_SPEED: f32 = 2.0;

pub(crate) use physics::*;
mod physics {
    /// How much does gravity affect weather.
    /// The gravity gradient comes from a poisson equation.
    /// We use a default stage gradient from 0.0 at the top to 1.0 at the
    /// bottom. This is multiplied by this constant to achieve the desired
    /// effect.
    pub(crate) const GRAVITY_MULTIPLIER: f32 = 8000.0;

    /// Caps gravity effect.
    /// If weather is falling faster than this it slows down.
    pub(crate) const TERMINAL_VELOCITY: f32 = -108.0;
    /// How fast does weather go from dip to terminal velocity.
    pub(crate) const SLOWDOWN_TO_TERMINAL_VELOCITY_FACTOR: f32 = 0.5;

    /// When left/right is pressed while up/down then weather gets an extra kick
    pub(crate) const HORIZONTAL_VELOCITY_BOOST_WHEN_JUMP_OR_DIP: f32 = 64.0;
    /// When down is pressed, weather's vertical velocity is set to this value
    pub(crate) const VERTICAL_VELOCITY_ON_DIP: f32 = -216.0;
    /// When left/right is pressed, weather gets an extra kick
    pub(crate) const DASH_VELOCITY_BOOST: f32 = 64.0;
    /// The jump function uses this to calculate the jump strength
    pub(crate) const VELOCITY_ON_JUMP: [f32; super::MAX_JUMPS] =
        [150.0, 140.0, 130.0, 120.0];
    /// When special is fired weather gets an extra kick in the chosen direction
    pub(crate) const VELOCITY_BOOST_ON_SPECIAL: f32 = 256.0;
}

pub(crate) use timings_of_actions::*;
mod timings_of_actions {
    use super::*;

    /// Pressing up does nothing if the last jump was less than this
    pub(crate) const MIN_JUMP_DELAY: Duration = from_millis(750);
    /// Pressing left/right does nothing if the last dash was less than this
    pub(crate) const MIN_DASH_DELAY: Duration = from_millis(500);
    /// Dashing in the opposite direction of the velocity should be available
    /// sooner that dashing in the same direction
    pub(crate) const MIN_DASH_AGAINST_VELOCITY_DELAY: Duration =
        from_millis(100);
    /// Pressing down does nothing if the last dip was less than this
    pub(crate) const MIN_DIP_DELAY: Duration = MIN_JUMP_DELAY;
    /// Maximum amount of time weather can be selecting the angle of its special
    /// before it fires
    pub(crate) const SPECIAL_LOADING_TIME: Duration = from_millis(750);
}

pub(crate) use timings_of_body_and_face_sprite_changes::*;
mod timings_of_body_and_face_sprite_changes {
    use super::*;

    /// Show the falling body sprite if appropriate at least after this long
    /// since the last body sprite change.
    /// This is override if dipped.
    pub(crate) const SHOW_FALLING_BODY_AFTER: Duration = from_millis(800);
    /// Show the falling face sprite if appropriate at least this long after the
    /// last the last face sprite change.
    pub(crate) const SHOW_FALLING_FACE_AFTER: Duration = from_millis(800);
    /// Set body to default values if no body change in at least
    /// this long after the last change sprite change
    pub(crate) const SHOW_DEFAULT_BODY_AFTER_IF_NO_CHANGE: Duration =
        from_millis(1000);
    /// Set face to default values if no _body_ change in at least
    /// this long after the last change sprite change
    pub(crate) const SHOW_DEFAULT_FACE_AFTER_IF_NO_BODY_CHANGE: Duration =
        from_millis(500);
    pub(crate) const SHOW_SPEARING_BODY_TOWARDS_FOR: Duration =
        from_millis(500);
    pub(crate) const SHOW_SPEARING_BODY_TOWARDS_IF_NO_CHANGE_FOR: Duration =
        from_millis(250);
}

pub(crate) use body_and_face_sprite_sizes::*;
mod body_and_face_sprite_sizes {
    use super::*;

    pub(crate) const BODY_ATLAS_ROWS: usize = 10;
    pub(crate) const BODY_ATLAS_COLS: usize = 10;
    /// We use padding because some sprites had artifacts from their neighbours.
    pub(crate) const BODY_ATLAS_PADDING: Vec2 = vec2(3.0, 3.0);
    pub(crate) const BODY_WIDTH: f32 = 35.0;
    pub(crate) const BODY_HEIGHT: f32 = 35.0;

    pub(crate) const FACE_ATLAS_PADDING: Vec2 = BODY_ATLAS_PADDING;
    pub(crate) const FACE_ATLAS_ROWS: usize = 5;
    pub(crate) const FACE_ATLAS_COLS: usize = 5;
    /// Note that this is the sprite size.
    /// The size of the actual visible face is smaller.
    /// It's surrounded by transparent pixels.
    pub(crate) const FACE_SPRITE_WIDTH: f32 = BODY_WIDTH;
    /// Note that this is the sprite size.
    /// The size of the actual visible face is smaller.
    /// It's surrounded by transparent pixels.
    pub(crate) const FACE_SPRITE_HEIGHT: f32 = BODY_HEIGHT;

    /// This is the size of the visible face.
    pub(crate) const FACE_RENDERED_SIZE: f32 = 15.0;
}

pub(crate) use spark_animation_on_special::*;
mod spark_animation_on_special {
    use super::*;

    pub(crate) const SPARK_FRAME_TIME: Duration = from_millis(75);
    pub(crate) const SPARK_FRAMES: usize = 10;

    pub(crate) const SPARK_SIDE: f32 = 90.0;

    pub(crate) const WHEN_LOADING_SPECIAL_STOP_MOVEMENT_WITHIN: Duration =
        from_millis(250);
    pub(crate) const START_SPARK_ANIMATION_AFTER_ELAPSED: Duration =
        from_millis(
            (SPECIAL_LOADING_TIME.as_millis()
                - SPARK_FRAME_TIME.as_millis() * 3) as u64,
        );
}

pub(crate) use camera_effects_on_special::*;
mod camera_effects_on_special {
    use super::*;

    pub(crate) const INITIAL_BLOOM_INTENSITY: f32 = 0.1; // start of special
    pub(crate) const PEAK_BLOOM_INTENSITY: f32 = 0.7; // special loaded

    pub(crate) const INITIAL_BLOOM_LFB: f32 = 0.25; // start of special
    pub(crate) const PEAK_BLOOM_LFB: f32 = 0.7; // special loaded

    /// While special is being loaded, we go from normal scale (1.0) to this.
    pub(crate) const ZOOM_IN_SCALE: f32 = 0.75;

    /// How long does the camera transition take to go from zoomed in to normal.
    pub(crate) const FROM_ZOOMED_BACK_TO_NORMAL_WHEN_SPECIAL_IS_LOADED_IN:
        Duration = from_millis(250);
    /// How long does the camera transition take to go from bloom to no bloom.
    pub(crate) const FADE_BLOOM_WHEN_SPECIAL_IS_LOADED_IN: Duration =
        from_millis(1500);
}

pub(crate) use arrow::*;
mod arrow {
    pub(crate) const ARROW_DISTANCE_FROM_EDGE: f32 = 25.0;
    /// The closer weather is the more the arrow is pushed back from the edge.
    pub(crate) const MAX_ARROW_PUSH_BACK: f32 = 15.0;
}

pub(crate) use light::*;
mod light {
    use bevy::math::Vec2;

    pub(crate) const OCCLUDER_SIZE: Vec2 = Vec2::new(3.0, 3.0);
}
