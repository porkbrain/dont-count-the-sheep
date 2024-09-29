use bevy::render::view::RenderLayers;
use common_loading_screen::{LoadingScreenSettings, LoadingScreenState};
use common_visuals::camera::render_layer;
use main_game_lib::cutscene::in_cutscene;
use top_down::{
    actor::{CharacterBundleBuilder, CharacterExt},
    inspect_and_interact::ZoneToInspectLabelEntity,
    TileMap,
};

use crate::prelude::*;

const THIS_SCENE: WhichTopDownScene = WhichTopDownScene::TwinpeaksApartment;

#[derive(TypePath, Default, Debug)]
struct TwinpeaksApartment;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(THIS_SCENE.loading()),
            bevy_rscn::return_start_loading_tscn_system::<TwinpeaksApartment>(
                format!("scenes/{}.tscn", THIS_SCENE.snake_case()),
            ),
        )
        .add_systems(
            Update,
            spawn
                .run_if(in_scene_loading_state(THIS_SCENE))
                .run_if(resource_exists::<TileMap>)
                .run_if(bevy_rscn::tscn_loaded_but_not_spawned::<
                    TwinpeaksApartment,
                >()),
        )
        .add_systems(OnExit(THIS_SCENE.leaving()), despawn)
        .add_systems(
            Update,
            exit.run_if(on_event::<TopDownAction>())
                .run_if(in_scene_running_state(THIS_SCENE))
                .run_if(not(in_cutscene())),
        );
    }
}

struct Spawner<'a> {
    player_entity: Entity,
    player_builder: &'a mut CharacterBundleBuilder,

    phoebe_entity: Entity,
    phoebe_builder: &'a mut CharacterBundleBuilder,
}

/// The names are stored in the scene file.
fn spawn(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut tscn: ResMut<Assets<TscnTree>>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,

    mut q: Query<&mut TscnTreeHandle<TwinpeaksApartment>>,
) {
    info!("Spawning {TwinpeaksApartment:?} scene");

    let tscn = q.single_mut().consume(&mut cmd, &mut tscn);
    let mut zone_to_inspect_label_entity = ZoneToInspectLabelEntity::default();

    let player = cmd.spawn_empty().id();
    let mut player_builder = common_story::Character::Winnie.bundle_builder();

    let phoebe = cmd.spawn_empty().id();
    let mut phoebe_builder = common_story::Character::Phoebe.bundle_builder();

    tscn.spawn_into(
        &mut cmd,
        &mut atlas_layouts,
        &asset_server,
        &mut TopDownTsncSpawner::new(
            &mut zone_to_inspect_label_entity,
            &mut Spawner {
                player_entity: player,
                player_builder: &mut player_builder,

                phoebe_entity: phoebe,
                phoebe_builder: &mut phoebe_builder,
            },
        ),
    );

    player_builder.insert_bundle_into(&asset_server, &mut cmd.entity(player));
    phoebe_builder.insert_bundle_into(&asset_server, &mut cmd.entity(phoebe));

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
            "TwinPeaks" => {
                cmd.entity(who).insert(LayoutEntity);
                cmd.entity(who).add_child(self.player_entity);
                cmd.entity(who).add_child(self.phoebe_entity);
            }
            "Entrance" => {
                let translation = ctx
                    .descriptions
                    .get(&who)
                    .expect("Missing description for {name}")
                    .translation;
                self.player_builder.initial_position(translation);
            }
            "PhoebeSpawn" => {
                let translation = ctx
                    .descriptions
                    .get(&who)
                    .expect("Missing description for {name}")
                    .translation;
                self.phoebe_builder.initial_position(translation);
            }
            _ => {}
        }
    }
}

fn exit(
    mut cmd: Commands,
    mut action_events: EventReader<TopDownAction>,
    mut transition: ResMut<GlobalGameStateTransition>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
    mut next_loading_screen_state: ResMut<NextState<LoadingScreenState>>,
) {
    let is_triggered = action_events
        .read()
        .any(|action| matches!(action, TopDownAction::Exit));

    if is_triggered {
        cmd.insert_resource(LoadingScreenSettings {
            atlas: Some(common_loading_screen::LoadingScreenAtlas::random()),
            stare_at_loading_screen_for_at_least: Some(from_millis(1000)),
            ..default()
        });

        next_loading_screen_state.set(common_loading_screen::start_state());

        *transition = GlobalGameStateTransition::TwinpeaksApartmentToDowntown;
        next_state.set(THIS_SCENE.leaving());
    }
}
