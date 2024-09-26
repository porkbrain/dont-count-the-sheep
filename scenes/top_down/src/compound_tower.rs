use bevy::render::view::RenderLayers;
use common_visuals::camera::render_layer;
use main_game_lib::{
    cutscene::in_cutscene, hud::notification::NotificationFifo,
    player_stats::PlayerStats,
};
use top_down::{
    actor::{CharacterBundleBuilder, CharacterExt},
    inspect_and_interact::ZoneToInspectLabelEntity,
    TileMap,
};

use crate::prelude::*;

const THIS_SCENE: WhichTopDownScene = WhichTopDownScene::CompoundTower;

#[derive(TypePath, Default, Debug)]
struct CompoundTower;

impl main_game_lib::bevy_rscn::TscnInBevy for CompoundTower {
    fn tscn_asset_path() -> String {
        format!("scenes/{}.tscn", THIS_SCENE.snake_case())
    }
}

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(THIS_SCENE.loading()),
            bevy_rscn::start_loading_tscn::<CompoundTower>,
        )
        .add_systems(
            Update,
            spawn
                .run_if(in_scene_loading_state(THIS_SCENE))
                .run_if(resource_exists::<TileMap>)
                .run_if(
                    bevy_rscn::tscn_loaded_but_not_spawned::<CompoundTower>(),
                ),
        )
        .add_systems(OnExit(THIS_SCENE.leaving()), despawn)
        .add_systems(
            Update,
            exit.run_if(on_event_variant(TopDownAction::Exit))
                .run_if(in_scene_running_state(THIS_SCENE))
                .run_if(not(in_cutscene())),
        );
    }
}

struct Spawner<'a> {
    player_entity: Entity,
    player_builder: &'a mut CharacterBundleBuilder,
}

/// The names are stored in the scene file.
fn spawn(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut tscn: ResMut<Assets<TscnTree>>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut notifications: ResMut<NotificationFifo>,
    mut player_stats: ResMut<PlayerStats>,

    mut q: Query<&mut TscnTreeHandle<CompoundTower>>,
) {
    info!("Spawning {CompoundTower:?} scene");
    player_stats.visited.compound_tower(&mut notifications);

    let tscn = q.single_mut().consume(&mut cmd, &mut tscn);
    let mut zone_to_inspect_label_entity = ZoneToInspectLabelEntity::default();

    let player = cmd.spawn_empty().id();
    let mut player_builder = common_story::Character::Winnie.bundle_builder();

    tscn.spawn_into(
        &mut cmd,
        &mut atlas_layouts,
        &asset_server,
        &mut TopDownTsncSpawner::new(
            &mut zone_to_inspect_label_entity,
            &mut Spawner {
                player_entity: player,
                player_builder: &mut player_builder,
            },
        ),
    );

    player_builder.insert_bundle_into(&asset_server, &mut cmd.entity(player));

    cmd.insert_resource(zone_to_inspect_label_entity);
}

fn despawn(mut cmd: Commands, root: Query<Entity, With<LayoutEntity>>) {
    debug!("Despawning layout entities");

    let root = root.single();
    cmd.entity(root).despawn_recursive();

    cmd.remove_resource::<ZoneToInspectLabelEntity>();
}

impl<'a> TscnSpawnHooks for Spawner<'a> {
    fn handle_2d_node(
        &mut self,
        cmd: &mut Commands,
        ctx: &mut SpawnerContext,
        _parent: Option<(Entity, NodeName)>,
        (who, NodeName(name)): (Entity, NodeName),
    ) {
        cmd.entity(who)
            .insert(RenderLayers::layer(render_layer::BG));

        match name.as_str() {
            "CompoundTower" => {
                cmd.entity(who).insert(LayoutEntity);
                cmd.entity(who).add_child(self.player_entity);
            }
            "Entrance" => {
                let translation = ctx
                    .descriptions
                    .get(&who)
                    .expect("Missing description for {name}")
                    .translation;
                self.player_builder.initial_position(translation);
            }
            _ => {}
        }
    }
}

fn exit(mut transition_params: TransitionParams) {
    transition_params.begin(GlobalGameStateTransition::TowerToCompound);
}
