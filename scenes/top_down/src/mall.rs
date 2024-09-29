use bevy::render::view::RenderLayers;
use bevy_grid_squared::Square;
use common_story::Character;
use common_visuals::camera::render_layer;
use main_game_lib::{
    cutscene::in_cutscene,
    hud::notification::NotificationFifo,
    player_stats::PlayerStats,
    top_down::{
        actor::BeginDialogEvent, layout::LAYOUT, npc::behaviors::PatrolSequence,
    },
};
use rand::prelude::SliceRandom;
use top_down::{
    actor::{CharacterBundleBuilder, CharacterExt},
    inspect_and_interact::ZoneToInspectLabelEntity,
    TileMap,
};

use crate::prelude::*;

const THIS_SCENE: WhichTopDownScene = WhichTopDownScene::Mall;

#[derive(TypePath, Default, Debug)]
struct Mall;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(THIS_SCENE.loading()),
            bevy_rscn::return_start_loading_tscn_system::<Mall>(format!(
                "scenes/{}.tscn",
                THIS_SCENE.snake_case()
            )),
        )
        .add_systems(
            Update,
            spawn
                .run_if(in_scene_loading_state(THIS_SCENE))
                .run_if(resource_exists::<TileMap>)
                .run_if(bevy_rscn::tscn_loaded_but_not_spawned::<Mall>()),
        )
        .add_systems(OnExit(THIS_SCENE.leaving()), despawn)
        .add_systems(
            Update,
            (exit, talk_to_ginger_cat)
                .run_if(on_event::<TopDownAction>())
                .run_if(in_scene_running_state(THIS_SCENE))
                .run_if(not(in_cutscene())),
        );
    }
}

struct Spawner<'a> {
    white_cat_entity: Entity,
    white_cat_patrol_points: &'a mut Vec<Square>,
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
        &mut cmd,
        &mut atlas_layouts,
        &asset_server,
        &mut TopDownTsncSpawner::new(
            &mut zone_to_inspect_label_entity,
            &mut Spawner {
                player_entity: player,
                white_cat_patrol_points: &mut white_cat_patrol_points,
                player_builder: &mut player_builder,
                white_cat_entity: white_cat,
            },
        ),
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
            "Mall" => {
                cmd.entity(who).insert(LayoutEntity);
                cmd.entity(who).add_child(self.player_entity);
                cmd.entity(who).add_child(self.white_cat_entity);
            }
            "Entrance" => {
                let translation = ctx
                    .descriptions
                    .get(&who)
                    .expect("Missing description for {name}")
                    .translation;
                self.player_builder.initial_position(translation);
            }
            s if s.starts_with("WhiteCatPatrolPoint") => {
                let translation = ctx
                    .descriptions
                    .get(&who)
                    .expect("Missing description for {name}")
                    .translation;
                self.white_cat_patrol_points
                    .push(LAYOUT.world_pos_to_square(translation));
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
        transition_params.begin(GlobalGameStateTransition::MallToDowntown);
    }
}

fn talk_to_ginger_cat(
    mut action_events: EventReader<TopDownAction>,
    mut begin_dialog_event: EventWriter<BeginDialogEvent>,
) {
    let is_triggered = action_events
        .read()
        .any(|action| matches!(action, TopDownAction::StartGingerCatDialog));

    if is_triggered {
        begin_dialog_event.send(BeginDialogEvent(Character::GingerCat.into()));
    }
}
