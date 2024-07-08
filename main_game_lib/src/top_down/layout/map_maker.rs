use std::{collections::BTreeMap, fs};

use bevy::{
    color::palettes::css::{GOLD, GREEN, RED},
    utils::HashSet,
    window::PrimaryWindow,
};
use bevy_egui::EguiContexts;
use bevy_grid_squared::{Square, SquareLayout};
use bevy_inspector_egui::{prelude::ReflectInspectorOptions, InspectorOptions};
use common_visuals::camera::MainCamera;
use ron::ser::PrettyConfig;
use serde::Serialize;
use smallvec::SmallVec;

use super::{
    build_pathfinding_graph::{GraphExt, LocalTileKindGraph},
    TileKind, TileMap, LAYOUT,
};
use crate::TopDownScene;

#[derive(Component)]
pub(crate) struct SquareSprite(Square);

#[derive(Resource, Reflect, InspectorOptions, Default)]
#[reflect(Resource, InspectorOptions)]
pub(crate) struct TileMapMakerToolbar {
    // these are configurable
    // ~
    // ~
    /// What kind of tile to paint.
    paint: TileKind,
    /// Each square has an associated list of tiles.
    /// Layer refers to the index in this list.
    /// We only manipulate the indexes of the tiles that equal to the
    /// `layer`.
    layer: usize,
    /// If set to true, will replace any tile kind.
    /// If set to false, will only paint over tiles that are `None`.
    paint_over_tiles: bool,

    // these are metadata used by the system
    // ~
    // ~
    /// We paint rectangles with this tool.
    /// When you click on a tile, it will start painting from there.
    /// When you release the mouse, it will stop painting and draw a
    /// rectangle of the `paint` kind from here to where you
    /// released the mouse.
    #[reflect(ignore)]
    begin_rect_at: Option<Square>,
    /// Copy of map is inserted when the map is loaded from fs, and then edited
    /// ONLY by the user map editing actions.
    /// No dynamic game logic should edit it.
    /// When we save the game, we store this map instead of the [`TileMap`]
    /// resource.
    ///
    /// We keep the copy in sync with the map resource in terms of the tiles
    /// being laid out in the same layers.
    #[reflect(ignore)]
    copy_of_map: HashMap<Square, SmallVec<[TileKind; 3]>>,
    /// If set to true, will display a grid on the map.
    /// If set to false, will not display a grid on the map.
    #[reflect(ignore)]
    display_grid: bool,
    /// Since we render tiles lazily, we need to keep track of which tiles
    /// have been rendered already to avoid rendering them again.
    #[reflect(ignore)]
    rendered_tiles: HashSet<Square>,
}

#[derive(Component)]
pub(crate) struct DebugLayoutGrid;

/// Contains:
/// 1. button to hide the grid with squares that show tile kinds
/// 2. button to store the map into a file
pub(crate) fn update_ui<T: TopDownScene>(
    mut contexts: EguiContexts,
    mut toolbar: ResMut<TileMapMakerToolbar>,
) {
    let ctx = contexts.ctx_mut();
    bevy_egui::egui::Window::new("Map maker")
        .vscroll(true)
        .show(ctx, |ui| {
            //
            // 1.
            //
            if ui.button("Toggle square grid").clicked() {
                toolbar.display_grid = !toolbar.display_grid;
            }

            //
            // 2.
            //
            if ui.button("Store map").clicked() {
                export_map::<T>(&mut toolbar);
            }
        });
}

pub(crate) fn spawn_debug_grid_root<T: TopDownScene>(mut cmd: Commands) {
    cmd.spawn((
        Name::new("Debug Layout Grid"),
        DebugLayoutGrid,
        SpatialBundle {
            transform: Transform::from_translation(Vec2::ZERO.extend(10.0)),
            ..default()
        },
    ));
}

/// We don't spawn and show all tiles because the map can be huge.
/// So we get the cursor position and show the tiles around it only.
pub(crate) fn show_tiles_around_cursor<T: TopDownScene>(
    mut cmd: Commands,
    map: Res<TileMap<T>>,
    mut toolbar: ResMut<TileMapMakerToolbar>,

    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    root: Query<(Entity, Option<&Children>), With<DebugLayoutGrid>>,
) {
    let (root, rendered_squares) = root.single();

    if !toolbar.display_grid {
        if let Some(rendered_squares) = rendered_squares {
            for rendered_square in rendered_squares.iter() {
                cmd.entity(*rendered_square).despawn_recursive();
            }
            toolbar.rendered_tiles.clear();
        }

        return;
    }

    let Some(clicked_at) = cursor_to_square(&LAYOUT, windows, cameras) else {
        return;
    };

    let [left, right, bottom, top] = T::bounds();

    // calculate a smaller rectangle +- 20 squares around the cursor
    let left = clicked_at.x.saturating_sub(20).clamp(left, right);
    let right = clicked_at.x.saturating_add(20).clamp(left, right);
    let bottom = clicked_at.y.saturating_sub(20).clamp(bottom, top);
    let top = clicked_at.y.saturating_add(20).clamp(bottom, top);

    for square in
        bevy_grid_squared::shapes::rectangle([left, right, bottom, top])
    {
        if !toolbar.rendered_tiles.insert(square) {
            // already present
            continue;
        }

        let world_pos = LAYOUT.square_to_world_pos(square);

        let kind = map
            .squares
            .get(&square)
            .and_then(|tiles| tiles.first()) // default to first layer
            .copied()
            .unwrap_or(TileKind::Empty);

        let child = cmd
            .spawn((SquareSprite(square), Name::new(format!("{square}"))))
            .insert(SpriteBundle {
                sprite: Sprite {
                    color: kind.color(),
                    // slightly smaller to show borders
                    custom_size: Some(LAYOUT.square() - 0.25),
                    ..default()
                },
                transform: Transform::from_translation(world_pos.extend(0.0)),
                ..default()
            })
            .id();
        cmd.entity(root).add_child(child);
    }
}

pub(crate) fn destroy_map<T: TopDownScene>(
    mut cmd: Commands,

    grid: Query<Entity, With<DebugLayoutGrid>>,
) {
    cmd.entity(grid.single()).despawn_recursive();
    cmd.remove_resource::<TileMapMakerToolbar>();
}

pub(crate) fn change_square_kind<T: TopDownScene>(
    mouse: Res<ButtonInput<MouseButton>>,
    mut map: ResMut<TileMap<T>>,
    mut toolbar: ResMut<TileMapMakerToolbar>,
    keyboard: Res<ButtonInput<KeyCode>>,

    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    if !toolbar.display_grid {
        return;
    }

    let ctrl_pressed = keyboard.pressed(KeyCode::ControlLeft);
    let esc_pressed = keyboard.just_pressed(KeyCode::Escape);
    let just_pressed_left = mouse.just_pressed(MouseButton::Left);
    let just_released_left = mouse.just_released(MouseButton::Left);
    let just_pressed_right = mouse.just_pressed(MouseButton::Right);

    // a) hold ctrl + press left to paint rect
    let start_painting_rect =
        ctrl_pressed && just_pressed_left && toolbar.begin_rect_at.is_none();
    // b) if painting rect, release left to stop painting
    let stop_painting_rect =
        toolbar.begin_rect_at.is_some() && just_released_left;
    // c) press right to paint single square (unless in rect mode)
    let paint_single_square =
        just_pressed_right && toolbar.begin_rect_at.is_none();
    // d) cancel painting rect on esc
    let cancel_painting =
        esc_pressed && !stop_painting_rect && toolbar.begin_rect_at.is_some();

    // if neither of these, then early return
    if !start_painting_rect
        && !stop_painting_rect
        && !paint_single_square
        && !cancel_painting
    {
        return;
    }

    let Some(clicked_at) = cursor_to_square(&LAYOUT, windows, cameras) else {
        return;
    };

    if start_painting_rect {
        toolbar.begin_rect_at = Some(clicked_at);
    } else if stop_painting_rect
        && let Some(begin_rect_at) = toolbar.begin_rect_at.take()
    {
        for square in selection_rect(begin_rect_at, clicked_at) {
            try_paint(&mut toolbar, &mut map, square);
        }
    } else if paint_single_square {
        try_paint(&mut toolbar, &mut map, clicked_at);
    } else if cancel_painting {
        toolbar.begin_rect_at.take();
    }
}

/// If a square can be painted, paint it.
fn try_paint<T: TopDownScene>(
    toolbar: &mut TileMapMakerToolbar,
    map: &mut TileMap<T>,
    at: Square,
) {
    if !T::contains(at) {
        return;
    }

    let tiles = map.squares.entry(at).or_default();
    if tiles.len() <= toolbar.layer {
        tiles.resize(toolbar.layer + 1, TileKind::Empty);
    }

    let can_paint =
        toolbar.paint_over_tiles || tiles[toolbar.layer] == TileKind::Empty;
    if !can_paint {
        return;
    }

    tiles[toolbar.layer] = toolbar.paint;
    // store the user change to the copy that will be saved
    let copy_entry = toolbar.copy_of_map.entry(at).or_default();
    if copy_entry.len() <= toolbar.layer {
        copy_entry.resize(toolbar.layer + 1, TileKind::Empty);
    }
    copy_entry[toolbar.layer] = toolbar.paint;
}

pub(crate) fn recolor_squares<T: TopDownScene>(
    map: ResMut<TileMap<T>>,
    toolbar: Res<TileMapMakerToolbar>,

    mut squares: Query<(&SquareSprite, &mut Sprite)>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    if !toolbar.display_grid {
        return;
    }

    let squares_painted: Option<HashSet<_>> =
        toolbar.begin_rect_at.and_then(|begin_rect_at| {
            let clicked_at = cursor_to_square(&LAYOUT, windows, camera)?;

            Some(selection_rect(begin_rect_at, clicked_at).collect())
        });

    for (SquareSprite(at), mut sprite) in squares.iter_mut() {
        let tile_kind = map
            .squares
            .get(at)
            .and_then(|tiles| tiles.get(toolbar.layer))
            .copied()
            .unwrap_or_default();

        // show where we're painting unless we're not allowed to
        // paint over tiles
        let color =
            if squares_painted.as_ref().map_or(false, |s| s.contains(at))
                && (toolbar.paint_over_tiles || tile_kind == TileKind::Empty)
            {
                toolbar.paint.color_selected()
            } else if tile_kind == toolbar.paint {
                tile_kind.color_selected()
            } else {
                tile_kind.color()
            };

        sprite.color = color;
    }
}

fn export_map<T: TopDownScene>(toolbar: &mut TileMapMakerToolbar) {
    if !toolbar.display_grid {
        return;
    }

    // filter out needless squares
    toolbar.copy_of_map.retain(|sq, tiles| {
        if !T::contains(*sq) {
            return false;
        }

        for tile in tiles.as_slice() {
            match tile {
                // Should not happen
                TileKind::Actor(_) => {
                    panic!("Actor tile found in toolbar map");
                }
                // these are fine
                TileKind::Wall | TileKind::Empty | TileKind::Trail => {}
                // fine for now but we might want to skip some of these in the
                // future
                TileKind::Local(_) => {}
            }
        }

        while tiles.ends_with(&[TileKind::Empty]) {
            tiles.pop();
        }

        !tiles.is_empty()
    });

    // equivalent to tile map, but sorted so that we can serialize it
    // and the output is deterministic
    //
    // this struct MUST serialize to a compatible ron output as TileMap
    #[derive(Serialize)]
    struct SortedTileMap {
        squares: BTreeMap<Square, SmallVec<[TileKind; 3]>>,
    }
    let tilemap_but_sorted = SortedTileMap {
        squares: toolbar.copy_of_map.clone().into_iter().collect(),
    };

    // for internal use only so who cares about unwraps and paths
    std::fs::write(
        format!("main_game/assets/maps/{}.ron", T::name()),
        ron::ser::to_string_pretty(
            &tilemap_but_sorted,
            PrettyConfig::default()
                .compact_arrays(true)
                .separate_tuple_members(false)
                .indentor(" ".to_string())
                .depth_limit(2),
        )
        .unwrap(),
    )
    .unwrap();

    let g = LocalTileKindGraph::compute_from(&TileMap::<T> {
        squares: toolbar.copy_of_map.clone(),
        zones: default(), // TODO
        _phantom: default(),
    });

    let scene_path =
        go_back_in_dir_tree_until_path_found(format!("scenes/{}", T::name()));

    let zone_tile_impl_rs = g.generate_zone_tile_impl_rs();
    fs::write(
        format!("{scene_path}/src/autogen/zone_tile_impl.rs"),
        zone_tile_impl_rs,
    )
    .unwrap();

    let dot_g = g.as_dotgraph(T::name());
    info!("Graphviz dot graph: \n{}", dot_g.as_dot());

    match dot_g.into_svg() {
        Ok(svg) => {
            fs::write(format!("{scene_path}/docs/tile-graph.svg"), svg)
                .unwrap();
        }
        Err(e) => {
            error!("Could not generate svg from dot graph: {e}");
        }
    }
}

impl TileKind {
    fn color(self) -> Color {
        match self {
            Self::Empty => Color::BLACK.with_alpha(0.25),
            Self::Wall => Color::BLACK.with_alpha(0.8),
            Self::Trail => Color::WHITE.with_alpha(0.25),
            Self::Actor { .. } => GOLD.with_alpha(0.25).into(),
            Self::Local(_) => RED.with_alpha(0.25).into(),
        }
    }

    fn color_selected(self) -> Color {
        match self {
            Self::Empty => Color::BLACK.with_alpha(0.25),
            Self::Wall => Color::BLACK.with_alpha(0.9),
            Self::Trail => Color::WHITE.with_alpha(0.5),
            // no point as it's not selectable
            Self::Actor { .. } => self.color(),
            Self::Local(_) => GREEN.with_alpha(0.25).into(),
        }
    }
}

fn selection_rect(
    begin_rect_at: Square,
    clicked_at: Square,
) -> impl ExactSizeIterator<Item = Square> {
    let left = begin_rect_at.x.min(clicked_at.x);
    let right = begin_rect_at.x.max(clicked_at.x);
    let bottom = begin_rect_at.y.min(clicked_at.y);
    let top = begin_rect_at.y.max(clicked_at.y);

    bevy_grid_squared::shapes::rectangle([left, right, bottom, top])
}

fn cursor_to_square(
    layout: &SquareLayout,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) -> Option<Square> {
    let cursor_pos = windows
        .get_single()
        .expect("some game window")
        .cursor_position()?;

    let (camera, camera_transform) = camera.single();
    let world_pos =
        camera.viewport_to_world_2d(camera_transform, cursor_pos)?;

    Some(layout.world_pos_to_square(world_pos))
}

fn go_back_in_dir_tree_until_path_found(mut path: String) -> String {
    const MAX_DEPTH: usize = 5;
    for _ in 0..MAX_DEPTH {
        if std::path::Path::new(&path).exists() {
            return path;
        }
        path = format!("../{path}");
    }

    panic!("Could not find path to {path}");
}

impl TileMapMakerToolbar {
    pub(crate) fn new(
        copy_of_map: HashMap<Square, SmallVec<[TileKind; 3]>>,
    ) -> Self {
        Self {
            copy_of_map,
            paint: default(),
            layer: 0,
            paint_over_tiles: false,
            display_grid: false,
            begin_rect_at: None,
            rendered_tiles: default(),
        }
    }
}
