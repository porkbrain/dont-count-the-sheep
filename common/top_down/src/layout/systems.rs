use bevy::prelude::*;
use common_assets::RonLoader;
use common_ext::QueryExt;

#[cfg(feature = "dev")]
use crate::layout::map_maker;
use crate::{
    actor::{self, player},
    ActorMovementEvent, TileKind, TileMap, TopDownScene,
};

/// Registers layout map for `T` where `T` is a type implementing
/// [`TopDownScene`]. This would be your level layout.
/// When [`crate::Actor`]s enter a zone within the map,
/// [`crate::ActorMovementEvent`] event is emitted.
///
/// If the `dev` feature is enabled, you can press `Enter` to export the map
/// to `map.ron` in the current directory.
/// We draw an overlay with tiles that you can edit with left and right mouse
/// buttons.
pub fn register<T: TopDownScene, S: States>(
    app: &mut App,
    loading: S,
    running: S,
) {
    app.add_event::<ActorMovementEvent<T::LocalTileKind>>()
        .init_asset_loader::<RonLoader<TileMap<T>>>()
        .init_asset::<TileMap<T>>()
        .register_type::<TileKind<T::LocalTileKind>>()
        .register_type::<TileMap<T>>()
        .register_type::<ActorMovementEvent<T::LocalTileKind>>();

    app.add_systems(OnEnter(loading.clone()), start_loading_map::<T>)
        .add_systems(
            First,
            try_insert_map_as_resource::<T>.run_if(in_state(loading)),
        )
        .add_systems(
            Update,
            actor::emit_movement_events::<T>
                .run_if(in_state(running.clone()))
                // so that we can emit this event on current frame
                .after(player::move_around::<T>),
        )
        .add_systems(OnExit(running.clone()), remove_resources::<T>);

    #[cfg(feature = "dev")]
    {
        use bevy::input::common_conditions::input_just_pressed;
        use bevy_inspector_egui::quick::ResourceInspectorPlugin;

        // we insert the toolbar along with the map
        app.register_type::<map_maker::TileMapMakerToolbar<T::LocalTileKind>>()
            .add_plugins(ResourceInspectorPlugin::<
                map_maker::TileMapMakerToolbar<T::LocalTileKind>,
            >::default());

        app.add_systems(
            OnEnter(running.clone()),
            map_maker::visualize_map::<T>,
        );
        app.add_systems(
            Update,
            (
                map_maker::change_square_kind::<T>,
                map_maker::recolor_squares::<T>,
            )
                .run_if(in_state(running.clone()))
                .chain(),
        );
        app.add_systems(
            Update,
            map_maker::export_map::<T>
                .run_if(input_just_pressed(KeyCode::Return))
                .run_if(in_state(running)),
        );
    }
}

/// Tells the game to start loading the map.
/// We need to keep checking for this to be done by calling
/// [`try_insert_map_as_resource`].
fn start_loading_map<T: TopDownScene>(
    mut cmd: Commands,
    assets: Res<AssetServer>,
) {
    let handle: Handle<TileMap<T>> = assets.load(T::asset_path());
    cmd.spawn(handle);
}

/// Run this to wait for the map to be loaded and insert it as a resource.
/// Call it after [`start_loading_map`].
/// Idempotent.
///
/// You should then check for the map as a resource in your systems and continue
/// with your game.
fn try_insert_map_as_resource<T: TopDownScene>(
    mut cmd: Commands,
    mut map_assets: ResMut<Assets<TileMap<T>>>,
    map: Query<(Entity, &Handle<TileMap<T>>)>,
) {
    let Some((entity, map)) = map.get_single_or_none() else {
        // if the map does no longer exist as a component handle, we either did
        // not spawn it or it's already a resource
        // the caller should check for the latter
        return;
    };

    // we cannot call remove straight away because panics - the handle is
    // removed, the map is not loaded yet and asset loader expects it to exist
    if map_assets.get(map).is_some() {
        let loaded_map = map_assets.remove(map).unwrap(); // safe ^

        #[cfg(feature = "dev")]
        {
            // include the loaded map in the toolbar, which will allow us to
            // store ONLY user changes, not dynamic changes made by the logic
            cmd.insert_resource(map_maker::TileMapMakerToolbar::new(
                loaded_map.squares.clone(),
            ));
        }

        cmd.insert_resource(loaded_map);
        cmd.init_resource::<actor::ActorZoneMap<T::LocalTileKind>>();
        cmd.entity(entity).despawn();
    }
}

fn remove_resources<T: TopDownScene>(mut cmd: Commands) {
    cmd.remove_resource::<TileMap<T>>();
    cmd.remove_resource::<actor::ActorZoneMap<T::LocalTileKind>>();

    #[cfg(feature = "dev")]
    {
        cmd.remove_resource::<map_maker::TileMapMakerToolbar<T::LocalTileKind>>();
    }
}
