//! Meditation is a vertical scroller minigame.
//! As the player, controlling a star named Hoshi, you are "falling" down.
//!
//! We split the game into rooms that can be chained one after another.
//! Each room's background is created in a way that its bottom edge matches
//! the top edge of the next room.
//!
//! ```text,no_run
//! |-----------------|    <- Entry room is always the same
//! |                 |
//! |  *              |
//! |                 |
//! +-----------------+    <- Randomly selected next room with top edge
//! |                 |       matching the bottom edge of the entry room
//! |                 |
//! |                 |
//! +-----------------+    <- Randomly selected next room with top edge
//! |                 |       matching the bottom edge of the previous room
//! |                 |
//!  ... ad infinitum
//! ```

use std::{collections::VecDeque, ops::DerefMut};

use bevy::{asset::AssetPath, render::view::RenderLayers, utils::HashMap};
use bevy_rscn::{
    return_start_loading_tscn_system, start_loading_tscn, NodeName,
    SpawnerContext, TscnSpawnHooks, TscnTree, TscnTreeHandle,
};
use common_visuals::camera::render_layer;
use main_game_lib::common_ext::QueryExt;

use crate::{
    consts::{DEFAULT_ROOM_HEIGHT_PX, ENTRY_ROOM_ASSET_PATH},
    hoshi,
    prelude::*,
};

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GlobalGameState::LoadingMeditation),
            // start the spawning process (the loading screen is already
            // started)
            return_start_loading_tscn_system::<RoomScene>(
                ENTRY_ROOM_ASSET_PATH,
            ),
        )
        .add_systems(
            Update,
            garbage_collect_old_rooms_and_spawn_new_ones
                .run_if(in_state(GlobalGameState::InGameMeditation)),
        )
        .add_systems(OnExit(GlobalGameState::QuittingMeditation), despawn);

        #[cfg(feature = "devtools")]
        {
            app.register_type::<RoomSpawner>()
                .register_type::<NextToLoad>()
                .register_type::<Room>()
                .register_type::<RoomScene>();
        }
    }
}

/// Marks a .tscn asset
#[cfg_attr(feature = "devtools", derive(Reflect))]
pub(crate) struct RoomScene;

/// Marks a room entity.
#[derive(Component)]
#[cfg_attr(feature = "devtools", derive(Reflect))]
struct Room;

/// Marks a [`TscnTreeHandle::<RoomScene>`] as the next room to spawn.
#[derive(Component)]
#[cfg_attr(feature = "devtools", derive(Reflect))]
struct NextToLoad;

/// Controls spawning of rooms based on where the player is.
#[derive(Resource, Default)]
#[cfg_attr(feature = "devtools", derive(Reflect))]
#[cfg_attr(feature = "devtools", reflect(Resource))]
struct RoomSpawner {
    /// Spawned room 2D entities, ordered by how they chain.
    ///
    /// The first one is the oldest still active room with the highest y
    /// coordinate.
    ///
    /// We despawn rooms that are no longer visible and above the player
    /// except for the last not visible one.
    active_rooms: VecDeque<Entity>,
    /// Maps .tscn asset paths to their loaded version.
    /// This avoids reparsing assets.
    /// We could also just store the handles and let the asset server deal with
    /// garbage collection, but this is fine I think.
    tscn_trees: HashMap<AssetPath<'static>, TscnTree>,
    /// The next room to spawn (by its path) if we know it yet and it's loaded.
    next_to_spawn: Option<AssetPath<'static>>,
    /// Vertical scroller, therefore this is an offset.
    ///
    /// Hopefully nobody would play long enough that float imprecision would be
    /// a concern.
    /// Or hopefully they would?
    y_offset_px: f32,
}

/// Given an entry room .tscn, spawns it and inserts a room spawner resource.
pub(crate) fn insert_room_spawner_resource_with_entry_room(
    cmd: &mut Commands,
    asset_server: &AssetServer,
    atlas_layouts: &mut Assets<TextureAtlasLayout>,
    tscn_tree: TscnTree,
) {
    let mut room_spawner = RoomSpawner::default();
    tscn_tree.spawn_into(cmd, atlas_layouts, asset_server, &mut room_spawner);
    debug_assert_eq!(room_spawner.active_rooms.len(), 1, "Only entry room");
    cmd.insert_resource(room_spawner);
}

impl TscnSpawnHooks for RoomSpawner {
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
            // this can be only present in the entry room scene
            "HoshiSpawn" => {
                info!("Hoshi spawn point");
                let translation = ctx
                    .descriptions
                    .get(&who)
                    .expect("HoshiSpawn node not present")
                    .translation;

                hoshi::spawn(cmd, ctx.asset_server, ctx.atlases, translation);
            }
            "MeditationRoomEntry" => {
                debug_assert!(
                    self.active_rooms.is_empty(),
                    "Entry room must be the first room"
                );

                // if we wanted to make rooms of different length, here's where
                // we'd take the length into consideration
                self.y_offset_px += DEFAULT_ROOM_HEIGHT_PX;
                self.active_rooms.push_back(who);
                cmd.entity(who).insert(Room);
            }
            s if s.starts_with("MeditationRoom") => {
                // update vertical offset of the room
                let desc = ctx
                    .descriptions
                    .get_mut(&who)
                    .expect("Room node not found");
                desc.translation.y += self.y_offset_px;

                // if we wanted to make rooms of different length, here's where
                // we'd take the length into consideration
                self.y_offset_px += DEFAULT_ROOM_HEIGHT_PX;
                self.active_rooms.push_back(who);
                cmd.entity(who).insert(Room);
            }
            _ => {}
        }
    }

    fn handle_plain_node(
        &mut self,
        cmd: &mut Commands,
        ctx: &mut SpawnerContext,
        _parent: (Entity, NodeName),
        (NodeName(who), node): (NodeName, bevy_rscn::RscnNode),
    ) {
        if who != "NextRooms" {
            panic!("Unexpected plain node: {who}");
        }

        let random_child_pos = rand::random::<usize>() % node.children.len();
        // SAFETY: we know that the random_child is within bounds
        let (NodeName(next_room_name_camel_case), _) =
            node.children.into_iter().nth(random_child_pos).unwrap();

        let asset_path = {
            let mut s =
                untools::camel_to_snake(&next_room_name_camel_case, false);
            s.push_str(".tscn");
            AssetPath::parse(&s).into_owned()
        };
        self.next_to_spawn = Some(asset_path.clone());

        if !self.tscn_trees.contains_key(&asset_path) {
            let (tree_handle_entity, _) = start_loading_tscn::<RoomScene>(
                cmd,
                ctx.asset_server,
                asset_path,
            );

            cmd.entity(tree_handle_entity).insert(NextToLoad);
        }
    }
}

fn despawn(
    mut cmd: Commands,

    rooms: Query<Entity, With<Room>>,
    tscn_tree_that_is_loading: Query<Entity, With<TscnTreeHandle<RoomScene>>>,
) {
    cmd.remove_resource::<RoomSpawner>();

    for room in rooms.iter().chain(tscn_tree_that_is_loading.iter()) {
        cmd.entity(room).despawn_recursive();
    }
}

/// System that manages the state of the rooms via [RoomSpawner].
///
/// 1. If there's anything being loaded right now, check if it's done and add it
///    to the room spawner cache
/// 2. If more than 3 top rooms are not visible anymore, despawn the topmost one
/// 3. There always should be two bottommost rooms that are not visible. If the
///    second to last room is visible, spawn the next room.
fn garbage_collect_old_rooms_and_spawn_new_ones(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut room_spawner: ResMut<RoomSpawner>,
    mut tscn_tree_assets: ResMut<Assets<TscnTree>>,
    mut atlases: ResMut<Assets<TextureAtlasLayout>>,

    spawned_rooms: Query<&ViewVisibility, With<Room>>,
    mut tscn_tree_that_is_loading: Query<
        &mut TscnTreeHandle<RoomScene>,
        With<NextToLoad>,
    >,
) {
    //
    // 1.
    //
    if let Some(mut tscn_tree_handle) =
        tscn_tree_that_is_loading.get_single_mut_or_none()
    {
        if tscn_tree_handle.is_loaded_with_dependencies(&asset_server) {
            // this also despawns the handle entity
            let tscn_tree =
                tscn_tree_handle.consume(&mut cmd, &mut tscn_tree_assets);

            let tscn_path = room_spawner
                .next_to_spawn
                .clone()
                .expect("Next room to spawn must be set if we're loading it");

            room_spawner.tscn_trees.insert(tscn_path, tscn_tree);
        }
    }

    //
    // 2.
    //
    let first_visible_room =
        room_spawner.active_rooms.iter().position(|room_entity| {
            let view_visibility = *spawned_rooms
                .get(*room_entity)
                .expect("Room entities must have ViewVisibility");
            // ie. it's shown
            view_visibility != ViewVisibility::HIDDEN
        });

    if let Some(first_visible_room) = first_visible_room {
        // We are running this system every tick.
        // The rate of spawning is _much_ slower than that.
        // We despawn one room per tick, because there is never going to be
        // a scenario where more than one rooms should be despawned at a time.
        // Even if, this system is called again in the next tick anyway.
        if first_visible_room > 3 {
            // SAFETY: we know there are at least 3 rooms
            let to_despawn = room_spawner.active_rooms.pop_front().unwrap();
            cmd.entity(to_despawn).despawn_recursive();
        }
    } else {
        // not sure about bevy behavior when minimizing a window etc.
        warn!("There are no visible rooms");
    }

    //
    // 3.
    //

    let should_spawn_next = room_spawner
        .active_rooms
        .iter()
        .rev() // from the bottommost room
        .nth(1) // second to last
        .map(|room_entity| {
            let view_visibility = *spawned_rooms
                .get(*room_entity)
                .expect("Room entities must have ViewVisibility");
            // ie. it's shown
            view_visibility != ViewVisibility::HIDDEN
        })
        .unwrap_or(true);

    if !should_spawn_next {
        return;
    }

    let Some(next_room_path) = room_spawner.next_to_spawn.clone() else {
        // We should never end up in this situation.
        // If the last spawned map was missing "NextRooms", we would panic in
        // the spawner.
        unreachable!("No next room to spawn");
    };

    let Some(tscn_tree) = room_spawner.tscn_trees.get(&next_room_path).cloned()
    else {
        return;
    };

    tscn_tree.spawn_into(
        &mut cmd,
        &mut atlases,
        &asset_server,
        room_spawner.deref_mut(),
    );
}
