use bevy::render::view::RenderLayers;
use main_game_lib::{
    common_top_down::{
        actor::CharacterExt,
        npc::{
            behaviors::IdlyWaiting, BehaviorLeaf, BehaviorNode, BehaviorTree,
            NpcInTheMap,
        },
        IntoMap,
    },
    common_visuals::camera::render_layer,
    vec2_ext::Vec2Ext,
};

use super::CharacterEntity;
use crate::{layout::HallwayEntity, prelude::*, Apartment};

pub(super) fn spawn(mut cmd: Commands) {
    cmd.spawn((
        CharacterEntity,
        HallwayEntity,
        RenderLayers::layer(render_layer::OBJ),
        NpcInTheMap::default(),
    ))
    .insert(
        common_story::Character::Marie
            .bundle_builder()
            .with_initial_position(vec2(-80.0, -100.0))
            .build::<Apartment>(),
    )
    .insert(BehaviorTree::new(ExampleBehavior));

    cmd.spawn((
        CharacterEntity,
        HallwayEntity,
        RenderLayers::layer(render_layer::OBJ),
        NpcInTheMap::default(),
    ))
    .insert(
        common_story::Character::Unnamed
            .bundle_builder()
            .with_initial_position(vec2(-150.0, -100.0))
            .build::<Apartment>(),
    )
    .insert(BehaviorTree::new(ExampleBehavior2));
}

struct ExampleBehavior;

impl From<ExampleBehavior> for BehaviorNode {
    fn from(_: ExampleBehavior) -> Self {
        let from = Apartment::layout().world_pos_to_square(
            vec2(490.0, 280.0).as_top_left_into_centered(),
        );
        let to = Apartment::layout().world_pos_to_square(
            vec2(470.0, 120.0).as_top_left_into_centered(),
        );

        BehaviorNode::Repeat(
            BehaviorNode::Sequence(vec![
                BehaviorNode::Leaf(BehaviorLeaf::FindPathToPosition(from)),
                IdlyWaiting(Duration::from_secs(1)).into(),
                BehaviorNode::Leaf(BehaviorLeaf::FindPathToPosition(to)),
                IdlyWaiting(Duration::from_secs(1)).into(),
            ])
            .into_boxed(),
        )
    }
}

struct ExampleBehavior2;

impl From<ExampleBehavior2> for BehaviorNode {
    fn from(_: ExampleBehavior2) -> Self {
        let from = Apartment::layout()
            .world_pos_to_square(vec2(84.0, 274.0).as_top_left_into_centered());
        let to = Apartment::layout().world_pos_to_square(
            vec2(160.0, 120.0).as_top_left_into_centered(),
        );

        BehaviorNode::Repeat(
            BehaviorNode::Sequence(vec![
                BehaviorNode::Leaf(BehaviorLeaf::FindPathToPosition(from)),
                IdlyWaiting(Duration::from_secs(1)).into(),
                BehaviorNode::Leaf(BehaviorLeaf::FindPathToPosition(to)),
                IdlyWaiting(Duration::from_secs(1)).into(),
            ])
            .into_boxed(),
        )
    }
}
