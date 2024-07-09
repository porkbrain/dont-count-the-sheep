use bevy::render::view::RenderLayers;
use bevy_grid_squared::{sq, GridDirection};
use common_story::Character;
use common_visuals::camera::{render_layer, MainCamera};
use main_game_lib::{
    cutscene::in_cutscene,
    hud::{daybar::UpdateDayBarEvent, notification::NotificationFifo},
    player_stats::PlayerStats,
    top_down::{layout::LAYOUT, scene_configs::ZoneTileKind},
};
use rscn::{NodeName, TscnSpawner, TscnTree, TscnTreeHandle};
use top_down::{
    actor::{CharacterBundleBuilder, CharacterExt},
    inspect_and_interact::ZoneToInspectLabelEntity,
    TileMap,
};

use crate::prelude::*;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(Compound::loading()),
            rscn::start_loading_tscn::<Compound>,
        )
        .add_systems(
            Update,
            spawn
                .run_if(Compound::in_loading_state())
                .run_if(resource_exists::<TileMap<Compound>>)
                .run_if(rscn::tscn_loaded_but_not_spawned::<Compound>()),
        )
        .add_systems(OnExit(Compound::quitting()), despawn)
        .add_systems(
            Update,
            (
                go_to_downtown
                    .run_if(on_event_variant(CompoundAction::GoToDowntown)),
                enter_tower
                    .run_if(on_event_variant(CompoundAction::EnterTower)),
            )
                .run_if(Compound::in_running_state())
                .run_if(not(in_cutscene())),
        );
    }
}

/// Assigned to the root of the scene.
/// We then recursively despawn it on scene leave.
#[derive(Component)]
pub(crate) struct LayoutEntity;

struct Spawner<'a> {
    player_entity: Entity,
    player_builder: &'a mut CharacterBundleBuilder,
    asset_server: &'a AssetServer,
    atlases: &'a mut Assets<TextureAtlasLayout>,
    zone_to_inspect_label_entity: &'a mut ZoneToInspectLabelEntity,
    camera_translation: &'a mut Vec3,
    daybar_event: &'a mut Events<UpdateDayBarEvent>,
    transition: GlobalGameStateTransition,
}

/// The names are stored in the scene file.
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
        &mut Spawner {
            player_entity: player,
            player_builder: &mut player_builder,
            asset_server: &asset_server,
            atlases: &mut atlas_layouts,
            zone_to_inspect_label_entity: &mut zone_to_inspect_label_entity,
            camera_translation: &mut camera.single_mut().translation,
            daybar_event: &mut daybar_event,

            transition: *transition,
        },
        &mut cmd,
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

impl<'a> TscnSpawner for Spawner<'a> {
    type LocalActionKind = CompoundAction;
    type LocalZoneKind = ZoneTileKind;

    fn on_spawned(
        &mut self,
        cmd: &mut Commands,
        who: Entity,
        NodeName(name): NodeName,
        translation: Vec3,
    ) {
        use GlobalGameStateTransition::*;

        let position = translation.truncate();
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

    fn add_texture_atlas(
        &mut self,
        layout: TextureAtlasLayout,
    ) -> Handle<TextureAtlasLayout> {
        self.atlases.add(layout)
    }

    fn load_texture(&mut self, path: &str) -> Handle<Image> {
        self.asset_server.load(path.to_owned())
    }

    fn map_zone_to_inspect_label_entity(
        &mut self,
        zone: Self::LocalZoneKind,
        entity: Entity,
    ) {
        self.zone_to_inspect_label_entity.insert(zone, entity);
    }
}

fn go_to_downtown(mut transition_params: TransitionParams) {
    transition_params.begin(GlobalGameStateTransition::CompoundToDowntown);
}

fn enter_tower(mut transition_params: TransitionParams) {
    transition_params.begin(GlobalGameStateTransition::CompoundToTower);
}
