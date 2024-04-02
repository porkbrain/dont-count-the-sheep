use bevy::prelude::*;
use common_ext::QueryExt;

#[cfg(feature = "devtools")]
use crate::top_down::layout::map_maker;
use crate::top_down::{TileMap, TopDownScene};

/// Tells the game to start loading the map.
/// We need to keep checking for this to be done by calling
/// [`try_insert_map_as_resource`].
pub(crate) fn start_loading_map<T: TopDownScene>(
    mut cmd: Commands,
    assets: Res<AssetServer>,
) {
    let asset_path = format!("maps/{}.ron", T::name());
    debug!("Loading map {} from {}", T::type_path(), asset_path);
    let handle: Handle<TileMap<T>> = assets.load(asset_path);
    cmd.spawn((Name::new(format!("TileMap<{}>", T::type_path())), handle));
}

/// Run this to wait for the map to be loaded and insert it as a resource.
/// Call it after [`start_loading_map`].
/// Idempotent.
///
/// You should then check for the map as a resource in your systems and continue
/// with your game.
pub(crate) fn try_insert_map_as_resource<T: TopDownScene>(
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

        #[cfg(feature = "devtools")]
        {
            // include the loaded map in the toolbar, which will allow us to
            // store ONLY user changes, not dynamic changes made by the logic
            cmd.insert_resource(map_maker::TileMapMakerToolbar::new(
                loaded_map.squares.clone(),
            ));
        }

        cmd.insert_resource(loaded_map);
        cmd.init_resource::<crate::top_down::actor::ActorZoneMap<T::LocalTileKind>>();
        cmd.entity(entity).despawn();
    }
}

pub(crate) fn remove_resources<T: TopDownScene>(mut cmd: Commands) {
    cmd.remove_resource::<TileMap<T>>();
    cmd.remove_resource::<crate::top_down::actor::ActorZoneMap<T::LocalTileKind>>();

    #[cfg(feature = "devtools")]
    {
        cmd.remove_resource::<map_maker::TileMapMakerToolbar<T::LocalTileKind>>();
    }
}
