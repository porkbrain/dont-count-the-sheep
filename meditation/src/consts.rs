pub(crate) const GRAVITY_PER_SECOND: f32 = 256.0;

pub(crate) mod weather {
    use std::time::Duration;

    pub(crate) const JUMP_ACCELERATION: f32 = super::GRAVITY_PER_SECOND / 4.0;
    pub(crate) const MIN_JUMP_DELAY: Duration = Duration::from_millis(100);
    pub(crate) const SPECIAL_LOADING_TIME: Duration = Duration::from_millis(1500);
    pub(crate) const MAX_JUMPS: u8 = 4;
}
