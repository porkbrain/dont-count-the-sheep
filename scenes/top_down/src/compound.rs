use bevy::render::view::RenderLayers;
use bevy_grid_squared::{sq, GridDirection};
use common_story::Character;
use common_visuals::camera::{render_layer, MainCamera};
use main_game_lib::{
    cutscene::in_cutscene,
    hud::{daybar::UpdateDayBarEvent, notification::NotificationFifo},
    player_stats::PlayerStats,
    top_down::layout::LAYOUT,
};
use top_down::{
    actor::{CharacterBundleBuilder, CharacterExt},
    inspect_and_interact::ZoneToInspectLabelEntity,
    TileMap,
};

use crate::prelude::*;

const THIS_SCENE: WhichTopDownScene = WhichTopDownScene::Compound;

#[derive(TypePath, Default, Debug)]
struct Compound;

impl main_game_lib::bevy_rscn::TscnInBevy for Compound {
    fn tscn_asset_path() -> String {
        format!("scenes/{}.tscn", THIS_SCENE.snake_case())
    }
}

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(THIS_SCENE.loading()),
            bevy_rscn::start_loading_tscn::<Compound>,
        )
        .add_systems(
            Update,
            spawn
                .run_if(in_scene_loading_state(THIS_SCENE))
                .run_if(resource_exists::<TileMap>)
                .run_if(bevy_rscn::tscn_loaded_but_not_spawned::<Compound>()),
        )
        .add_systems(OnExit(THIS_SCENE.leaving()), despawn)
        .add_systems(
            Update,
            (
                go_to_downtown
                    .run_if(on_event_variant(TopDownAction::GoToDowntown)),
                enter_tower.run_if(on_event_variant(TopDownAction::EnterTower)),
            )
                .run_if(in_scene_running_state(THIS_SCENE))
                .run_if(not(in_cutscene())),
        );
    }
}

struct Spawner<'a> {
    player_entity: Entity,
    player_builder: &'a mut CharacterBundleBuilder,
    camera_translation: &'a mut Vec3,
    daybar_event: &'a mut Events<UpdateDayBarEvent>,
    transition: GlobalGameStateTransition,
}

/// The names are stored in the scene file.
#[allow(clippy::too_many_arguments)]
fn spawn(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut tscn: ResMut<Assets<TscnTree>>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    transition: Res<GlobalGameStateTransition>,
    mut daybar_event: ResMut<Events<UpdateDayBarEvent>>,
    mut notifications: ResMut<NotificationFifo>,
    mut player_stats: ResMut<PlayerStats>,

    mut camera: Query<&mut Transform, With<MainCamera>>,
    mut q: Query<&mut TscnTreeHandle<Compound>>,
) {
    info!("Spawning {Compound:?} scene");
    player_stats.visited.compound(&mut notifications);

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
                camera_translation: &mut camera.single_mut().translation,
                daybar_event: &mut daybar_event,
                transition: *transition,
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
        use GlobalGameStateTransition::*;

        let position = descriptions
            .get(&who)
            .expect("Missing description for {name}")
            .translation;

        cmd.entity(who)
            .insert(RenderLayers::layer(render_layer::BG));

        match (name.as_str(), self.transition) {
            ("Compound", _) => {
                cmd.entity(who).insert(LayoutEntity);
                cmd.entity(who).add_child(self.player_entity);
            }
            ("MainGate", DowntownToCompound)
            | ("TowerEntrance", TowerToCompound) => {
                let face_up = name.as_str() == "MainGate";

                self.camera_translation.x = position.x;
                self.camera_translation.y = position.y;
                // we multiply by 4 because winnie is walking across 2 tiles and
                // we want her to be extra extra slow because it looks better
                self.player_builder
                    .initial_step_time(Character::Winnie.slow_step_time() * 4);
                self.player_builder.initial_position(position);
                self.player_builder.walking_to(top_down::ActorTarget::new(
                    LAYOUT.world_pos_to_square(position)
                        + sq(0, 2 * if face_up { 1 } else { -1 }),
                ));
                if face_up {
                    self.player_builder.initial_direction(GridDirection::Top);
                }

                self.daybar_event.send(UpdateDayBarEvent::ChangedScene);
            }

            _ => {}
        }
    }
}

fn go_to_downtown(mut transition_params: TransitionParams) {
    transition_params.begin(GlobalGameStateTransition::CompoundToDowntown);
}

fn enter_tower(mut transition_params: TransitionParams) {
    transition_params.begin(GlobalGameStateTransition::CompoundToTower);
}
