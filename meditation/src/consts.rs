pub(crate) const GRAVITY_PER_SECOND: f32 = 256.0;

pub(crate) mod weather {
    use std::time::Duration;

    /// After jumping, weather gets accelerated by this much up.
    /// Existing acceleration is overwritten.
    pub(crate) const JUMP_ACCELERATION: f32 = super::GRAVITY_PER_SECOND / 4.0;
    /// Pressing jump won't do anything if the last jump was less than this
    pub(crate) const MIN_JUMP_DELAY: Duration = Duration::from_millis(150);
    /// Maximum amount of time weather can be selecting the angle of its special
    /// before it fires.
    pub(crate) const SPECIAL_LOADING_TIME: Duration =
        Duration::from_millis(1500);
    /// Cannot jump more times in a row than this before resetting.
    pub(crate) const MAX_JUMPS: u8 = 4;
    /// When left/right is pressed while jumping weather gets an extra kick
    pub(crate) const HORIZONTAL_VELOCITY_BOOST_WHEN_JUMPING: f32 = 10.0;
}
