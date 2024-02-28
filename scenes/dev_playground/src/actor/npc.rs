use bevy::render::view::RenderLayers;
use common_top_down::{
    actor::CharacterExt,
    npc::{
        behaviors::IdlyWaiting, BehaviorLeaf, BehaviorNode, BehaviorTree,
        NpcInTheMap,
    },
};
use common_visuals::camera::render_layer;
use main_game_lib::vec2_ext::Vec2Ext;

use super::CharacterEntity;
use crate::{prelude::*, DevPlayground};

pub(super) fn spawn(mut cmd: Commands, asset_server: Res<AssetServer>) {
    common_story::Character::Marie
        .bundle_builder()
        .with_initial_position(vec2(-80.0, -100.0))
        .insert::<DevPlayground>(
            &asset_server,
            &mut cmd.spawn((
                CharacterEntity,
                RenderLayers::layer(render_layer::OBJ),
                NpcInTheMap::default(),
                BehaviorTree::new(ExampleBehavior),
            )),
        );

    common_story::Character::Unnamed
        .bundle_builder()
        .with_initial_position(vec2(-150.0, -100.0))
        .insert::<DevPlayground>(
            &asset_server,
            &mut cmd.spawn((
                CharacterEntity,
                RenderLayers::layer(render_layer::OBJ),
                NpcInTheMap::default(),
                BehaviorTree::new(ExampleBehavior2),
            )),
        );
}

struct ExampleBehavior;

impl From<ExampleBehavior> for BehaviorNode {
    fn from(_: ExampleBehavior) -> Self {
        let from = DevPlayground::layout().world_pos_to_square(
            vec2(490.0, 280.0).as_top_left_into_centered(),
        );
        let to = DevPlayground::layout().world_pos_to_square(
            vec2(470.0, 120.0).as_top_left_into_centered(),
        );

        BehaviorNode::Repeat(
            BehaviorNode::Sequence(vec![
                BehaviorNode::Leaf(BehaviorLeaf::find_path_to(from)),
                IdlyWaiting(Duration::from_secs(1)).into(),
                BehaviorNode::Leaf(BehaviorLeaf::find_path_to(to)),
                IdlyWaiting(Duration::from_secs(1)).into(),
            ])
            .into_boxed(),
        )
    }
}

struct ExampleBehavior2;

impl From<ExampleBehavior2> for BehaviorNode {
    fn from(_: ExampleBehavior2) -> Self {
        let from = DevPlayground::layout()
            .world_pos_to_square(vec2(84.0, 274.0).as_top_left_into_centered());
        let to = DevPlayground::layout().world_pos_to_square(
            vec2(160.0, 120.0).as_top_left_into_centered(),
        );

        BehaviorNode::Repeat(
            BehaviorNode::Sequence(vec![
                BehaviorNode::Leaf(BehaviorLeaf::find_path_to(from)),
                IdlyWaiting(Duration::from_secs(1)).into(),
                BehaviorNode::Leaf(BehaviorLeaf::find_path_to(to)),
                IdlyWaiting(Duration::from_secs(1)).into(),
            ])
            .into_boxed(),
        )
    }
}
