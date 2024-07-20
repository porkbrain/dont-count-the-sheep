use bevy::render::view::RenderLayers;
use bevy_grid_squared::Square;
use common_story::Character;
use common_visuals::camera::render_layer;
use main_game_lib::{
    cutscene::in_cutscene,
    hud::notification::NotificationFifo,
    player_stats::PlayerStats,
    top_down::{
        actor::BeginDialogEvent, layout::LAYOUT,
        npc::behaviors::PatrolSequence, scene_configs::ZoneTileKind,
    },
};
use rand::prelude::SliceRandom;
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
            OnEnter(THIS_SCENE.loading()),
            rscn::start_loading_tscn::<Mall>,
        )
        .add_systems(
            Update,
            spawn
                .run_if(in_scene_loading_state(THIS_SCENE))
                .run_if(resource_exists::<TileMap<Mall>>)
                .run_if(rscn::tscn_loaded_but_not_spawned::<Mall>()),
        )
        .add_systems(OnExit(THIS_SCENE.leaving()), despawn)
        .add_systems(
            Update,
            (exit, talk_to_ginger_cat)
                .run_if(on_event::<MallAction>())
                .run_if(in_scene_running_state(THIS_SCENE))
                .run_if(not(in_cutscene())),
        );
    }
}

/// Assigned to the root of the scene.
/// We then recursively despawn it on scene leave.
#[derive(Component)]
pub(crate) struct LayoutEntity;

struct Spawner<'a> {
    white_cat_entity: Entity,
    white_cat_patrol_points: &'a mut Vec<Square>,
    player_entity: Entity,
    player_builder: &'a mut CharacterBundleBuilder,
    asset_server: &'a AssetServer,
    atlases: &'a mut Assets<TextureAtlasLayout>,
    zone_to_inspect_label_entity: &'a mut ZoneToInspectLabelEntity,
}

/// The names are stored in the scene file.
fn spawn(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut tscn: ResMut<Assets<TscnTree>>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut notifications: ResMut<NotificationFifo>,
    mut player_stats: ResMut<PlayerStats>,

    mut q: Query<&mut TscnTreeHandle<Mall>>,
) {
    info!("Spawning Mall scene");
    player_stats.visited.mall(&mut notifications);

    let tscn = q.single_mut().consume(&mut cmd, &mut tscn);
    let mut zone_to_inspect_label_entity = ZoneToInspectLabelEntity::default();

    let player = cmd.spawn_empty().id();
    let mut player_builder = common_story::Character::Winnie.bundle_builder();

    let white_cat = cmd.spawn_empty().id();
    let mut white_cat_builder =
        common_story::Character::WhiteCat.bundle_builder();
    let mut white_cat_patrol_points = Vec::new();

    tscn.spawn_into(
        &mut Spawner {
            player_entity: player,
            white_cat_patrol_points: &mut white_cat_patrol_points,
            player_builder: &mut player_builder,
            white_cat_entity: white_cat,
            asset_server: &asset_server,
            atlases: &mut atlas_layouts,
            zone_to_inspect_label_entity: &mut zone_to_inspect_label_entity,
        },
        &mut cmd,
    );

    cmd.insert_resource(zone_to_inspect_label_entity);

    player_builder.insert_bundle_into(&asset_server, &mut cmd.entity(player));

    let points = {
        let mut rng = rand::thread_rng();
        assert!(
            !white_cat_patrol_points.is_empty(),
            "No patrol points for white cat"
        );

        // double the patrol points for greater variety
        white_cat_patrol_points.extend(white_cat_patrol_points.clone());

        // shuffle them
        white_cat_patrol_points.shuffle(&mut rng);

        white_cat_patrol_points
    };
    // SAFETY: we just checked that the vec is not empty
    white_cat_builder
        .initial_square(points.last().copied().unwrap())
        .behavior_tree(PatrolSequence {
            wait_at_each: from_millis(10_000),
            points,
        });
    white_cat_builder
        .insert_bundle_into(&asset_server, &mut cmd.entity(white_cat));
}

fn despawn(mut cmd: Commands, root: Query<Entity, With<LayoutEntity>>) {
    debug!("Despawning layout entities");

    let root = root.single();
    cmd.entity(root).despawn_recursive();

    cmd.remove_resource::<ZoneToInspectLabelEntity>();
}

impl<'a> TscnSpawner for Spawner<'a> {
    type LocalActionKind = MallAction;
    type ZoneKind = ZoneTileKind;

    fn on_spawned(
        &mut self,
        cmd: &mut Commands,
        who: Entity,
        NodeName(name): NodeName,
        translation: Vec3,
    ) {
        cmd.entity(who)
            .insert(RenderLayers::layer(render_layer::BG));

        match name.as_str() {
            "Mall" => {
                cmd.entity(who).insert(LayoutEntity);
                cmd.entity(who).add_child(self.player_entity);
                cmd.entity(who).add_child(self.white_cat_entity);
            }
            "Entrance" => {
                self.player_builder.initial_position(translation.truncate());
            }
            s if s.starts_with("WhiteCatPatrolPoint") => {
                self.white_cat_patrol_points
                    .push(LAYOUT.world_pos_to_square(translation.truncate()));
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
        zone: Self::ZoneKind,
        entity: Entity,
    ) {
        self.zone_to_inspect_label_entity.insert(zone, entity);
    }
}

fn exit(
    mut transition_params: TransitionParams,
    mut action_events: EventReader<MallAction>,
) {
    let is_triggered = action_events
        .read()
        .any(|action| matches!(action, MallAction::ExitScene));

    if is_triggered {
        transition_params.begin(GlobalGameStateTransition::MallToDowntown);
    }
}

fn talk_to_ginger_cat(
    mut action_events: EventReader<MallAction>,
    mut begin_dialog_event: EventWriter<BeginDialogEvent>,
) {
    let is_triggered = action_events
        .read()
        .any(|action| matches!(action, MallAction::StartGingerCatDialog));

    if is_triggered {
        begin_dialog_event.send(BeginDialogEvent(Character::GingerCat.into()));
    }
}
