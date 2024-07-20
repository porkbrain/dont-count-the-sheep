use crate::prelude::*;

/// Assigned to the root of the scene.
/// We then recursively despawn it on scene leave.
#[derive(Component)]
pub(crate) struct LayoutEntity;
