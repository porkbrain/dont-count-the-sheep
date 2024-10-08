use bevy::render::view::RenderLayers;
use bevy_rscn::{NodeName, TscnSpawnHooks, TscnTree, TscnTreeHandle};
use common_visuals::camera::render_layer;
use main_game_lib::{
    cutscene::in_cutscene, hud::notification::NotificationFifo,
    player_stats::PlayerStats,
    top_down::environmental_objects::door::DoorBuilder,
};
use top_down::{
    actor::{CharacterBundleBuilder, CharacterExt},
    inspect_and_interact::ZoneToInspectLabelEntity,
    TileMap,
};

use crate::prelude::*;

const THIS_SCENE: WhichTopDownScene = WhichTopDownScene::Clinic;

#[derive(TypePath, Default, Debug)]
struct Clinic;

impl main_game_lib::bevy_rscn::TscnInBevy for Clinic {
    fn tscn_asset_path() -> String {
        format!("scenes/{}.tscn", THIS_SCENE.snake_case())
    }
}

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(THIS_SCENE.loading()),
            bevy_rscn::start_loading_tscn::<Clinic>,
        )
        .add_systems(
            Update,
            spawn
                .run_if(in_scene_loading_state(THIS_SCENE))
                .run_if(resource_exists::<TileMap>)
                .run_if(bevy_rscn::tscn_loaded_but_not_spawned::<Clinic>()),
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
}

/// The names are stored in the scene file.
fn spawn(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut tscn: ResMut<Assets<TscnTree>>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut notifications: ResMut<NotificationFifo>,
    mut player_stats: ResMut<PlayerStats>,

    mut q: Query<&mut TscnTreeHandle<Clinic>>,
) {
    info!("Spawning {Clinic:?} scene");
    player_stats.visited.clinic(&mut notifications);

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
        descriptions: &mut EntityDescriptionMap,
        _parent: Option<(Entity, NodeName)>,
        (who, NodeName(name)): (Entity, NodeName),
    ) {
        cmd.entity(who)
            .insert(RenderLayers::layer(render_layer::BG));

        match name.as_str() {
            "Clinic" => {
                cmd.entity(who).insert(LayoutEntity);
                cmd.entity(who).add_child(self.player_entity);
            }
            "Entrance" => {
                let translation = descriptions
                    .get(&who)
                    .expect("Missing description for {name}")
                    .translation;
                self.player_builder.initial_position(translation);
            }
            "Door" => {
                let door = DoorBuilder::new(ZoneTileKind::Door).build();
                cmd.entity(who).insert(door);
            }
            _ => {}
        }
    }
}

fn exit(
    mut transition_params: TransitionParams,
    mut action_events: EventReader<TopDownAction>,
) {
    let is_triggered = action_events
        .read()
        .any(|action| matches!(action, TopDownAction::Exit));

    if is_triggered {
        transition_params.begin(GlobalGameStateTransition::ClinicToDowntown);
    }
}
