use bevy::render::view::RenderLayers;
use common_top_down::{
    actor::CharacterExt,
    layout::LAYOUT,
    npc::{behaviors::IdlyWaiting, BehaviorLeaf, BehaviorNode, BehaviorTree},
};
use common_visuals::camera::render_layer;
use main_game_lib::vec2_ext::Vec2Ext;

use crate::{layout::HallwayEntity, prelude::*, Apartment};

pub(crate) fn spawn(
    cmd: &mut Commands,
    asset_server: &AssetServer,
) -> Vec<Entity> {
    let mut marie = cmd.spawn((
        BehaviorTree::new(ExampleBehavior),
        HallwayEntity,
        RenderLayers::layer(render_layer::OBJ),
    ));
    common_story::Character::Marie
        .bundle_builder()
        .with_sprite_color(Some(PRIMARY_COLOR))
        .with_initial_position(vec2(-80.0, -100.0))
        .insert::<Apartment>(asset_server, &mut marie);
    let marie = marie.id();

    let mut unnamed = cmd.spawn((
        HallwayEntity,
        BehaviorTree::new(ExampleBehavior2),
        RenderLayers::layer(render_layer::OBJ),
    ));
    common_story::Character::Unnamed
        .bundle_builder()
        .with_sprite_color(Some(PRIMARY_COLOR))
        .with_initial_position(vec2(-150.0, -100.0))
        .insert::<Apartment>(asset_server, &mut unnamed);
    let unnamed = unnamed.id();

    let mut bolt = cmd.spawn((RenderLayers::layer(render_layer::OBJ),));
    common_story::Character::Bolt
        .bundle_builder()
        .with_initial_position(vec2(180.0, 35.0))
        .insert::<Apartment>(asset_server, &mut bolt);
    let bolt = bolt.id();

    vec![marie, unnamed, bolt]
}

struct ExampleBehavior;

impl From<ExampleBehavior> for BehaviorNode {
    fn from(_: ExampleBehavior) -> Self {
        let from = LAYOUT.world_pos_to_square(
            vec2(490.0, 280.0).as_top_left_into_centered(),
        );
        let to = LAYOUT.world_pos_to_square(
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
        let from = LAYOUT
            .world_pos_to_square(vec2(84.0, 274.0).as_top_left_into_centered());
        let to = LAYOUT.world_pos_to_square(
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
