use bevy::render::view::RenderLayers;
use bevy_magic_light_2d::gi::LightScene;

use crate::prelude::*;

#[derive(Component, Default, Clone, TypePath)]
pub(crate) struct BackgroundLightScene;

impl LightScene for BackgroundLightScene {
    const HANDLE_START: u128 = 23475629871623176235;

    fn render_layer_index() -> u8 {
        (RenderLayers::TOTAL_LAYERS - 2) as u8
    }

    fn camera_order() -> isize {
        1
    }
}

#[derive(Component, Default, Clone, TypePath)]
struct ObjectsLightScene;

impl LightScene for ObjectsLightScene {
    const HANDLE_START: u128 = 6582023275553590181;

    fn render_layer_index() -> u8 {
        (RenderLayers::TOTAL_LAYERS - 3) as u8
    }

    fn camera_order() -> isize {
        -1
    }
}

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        BackgroundLightScene::build(app);
        ObjectsLightScene::build(app);
    }

    fn finish(&self, app: &mut App) {
        BackgroundLightScene::finish(app);
        ObjectsLightScene::finish(app);
    }
}
