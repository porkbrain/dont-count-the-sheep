use bevy::render::view::RenderLayers;
use common_visuals::camera::render_layer;
use main_game_lib::{
    common_ext::QueryExt,
    cutscene::{enter_dark_door::EnterDarkDoor, in_cutscene, IntoCutscene},
};
use top_down::{
    actor::{CharacterBundleBuilder, CharacterExt},
    inspect_and_interact::ZoneToInspectLabelEntity,
    TileMap,
};

use crate::prelude::*;

const THIS_SCENE: WhichTopDownScene = WhichTopDownScene::Building1Basement2;

#[derive(TypePath, Default, Debug)]
struct Building1Basement2;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(THIS_SCENE.loading()),
            bevy_rscn::return_start_loading_tscn_system::<Building1Basement2>(
                format!("scenes/{}.tscn", THIS_SCENE.snake_case()),
            ),
        )
        .add_systems(
            Update,
            spawn
                .run_if(in_scene_loading_state(THIS_SCENE))
                .run_if(resource_exists::<TileMap>)
                .run_if(bevy_rscn::tscn_loaded_but_not_spawned::<
                    Building1Basement2,
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

/// The door sprite that leads to the storage basement.
#[derive(Component)]
struct DoorToStorageBasement;

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

    mut q: Query<&mut TscnTreeHandle<Building1Basement2>>,
) {
    info!("Spawning {Building1Basement2:?} scene");

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
            "Building1Basement2" => {
                cmd.entity(who).insert(LayoutEntity);
                cmd.entity(who).add_child(self.player_entity);
            }
            "BasementDoor" => {
                cmd.entity(who).insert(DoorToStorageBasement);
            }
            "Exit" => {
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

fn exit(
    mut cmd: Commands,
    mut action_events: EventReader<TopDownAction>,

    player: Query<Entity, With<Player>>,
    door: Query<Entity, With<DoorToStorageBasement>>,
    points: Query<(&Name, &bevy_rscn::Point)>,
) {
    let is_triggered = action_events
        .read()
        .any(|action| matches!(action, TopDownAction::Exit));

    if is_triggered {
        let Some(player) = player.get_single_or_none() else {
            return;
        };

        let door_entrance = points
            .iter()
            .find_map(|(name, bevy_rscn::Point(pos))| {
                if name == &Name::new("Exit") {
                    Some(*pos)
                } else {
                    None
                }
            })
            .expect("Missing point for Exit");

        EnterDarkDoor {
            player,
            door: door.single(),
            door_entrance,
            change_global_state_to: THIS_SCENE.leaving(),
            transition:
                GlobalGameStateTransition::Building1Basement2ToBasement1,
            loading_screen: default(),
        }
        .spawn(&mut cmd);
    }
}
