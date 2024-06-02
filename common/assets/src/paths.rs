//! Some assets that the game loads are stored as string paths here.
//! More are stored in scene RON files.

pub mod fonts {
    pub const FOLDER: &str = "fonts";

    pub const PIXEL1: &str = "fonts/shaperka.ttf";
    pub const PENCIL1: &str = "fonts/pencil.ttf";
    pub const TINY_PIXEL1: &str = "fonts/tiny_pixel.ttf";
}

pub mod meditation {
    pub const FOLDER: &str = "meditation";

    pub const BACKGROUND_DEFAULT: &str = "meditation/textures/bg/default.png";
    pub const SHOOTING_STAR_ATLAS: &str =
        "meditation/textures/bg/shootingstar_atlas.png";
    pub fn twinkle(i: usize) -> String {
        format!("meditation/textures/bg/twinkle{i}.png")
    }

    pub const CLIMATE_DEFAULT: &str = "meditation/textures/climate/default.png";

    pub const HOSHI_ARROW: &str = "meditation/textures/hoshi/arrow.png";
    pub const BODY_ATLAS: &str = "meditation/textures/hoshi/body_atlas.png";
    pub const SPARK_ATLAS: &str = "meditation/textures/hoshi/spark_atlas.png";
    pub const FACE_ATLAS: &str = "meditation/textures/hoshi/face_atlas.png";

    pub const BLACKHOLE_ATLAS: &str =
        "meditation/textures/polpo/blackhole_atlas.png";
    pub const BLACKHOLE_FLICKER: &str =
        "meditation/textures/polpo/blackhole_flicker.png";
    pub const TV_STATIC_ATLAS: &str =
        "meditation/textures/polpo/static_atlas.png";
    pub const CRACK_ATLAS: &str = "meditation/textures/polpo/crack_atlas.png";
    pub const POLPO_FRAME: &str = "meditation/textures/polpo/frame.png";
    pub const BOLT: &str = "meditation/textures/polpo/bolt.png";
    pub const TENTACLE_ATLAS: &str =
        "meditation/textures/polpo/tentacle_atlas.png";

    pub const MENU_BOX: &str = "meditation/ui/menu_box.png";
    pub const FACE_ON_CONTINUE: &str = "meditation/ui/face_on_continue.png";
    pub const FACE_ON_RESTART: &str = "meditation/ui/face_on_restart.png";
    pub const FACE_ON_EXIT: &str = "meditation/ui/face_on_exit.png";

    pub const VIDEO_ALEX: &str = "meditation/textures/polpo/videos/alex.webp";
    pub const VIDEO_BUNNY: &str = "meditation/textures/polpo/videos/bunny.webp";
    pub const VIDEO_DANCE: &str = "meditation/textures/polpo/videos/dance.webp";
    pub const VIDEO_FRAGRANCE: &str =
        "meditation/textures/polpo/videos/fragrance.webp";
    pub const VIDEO_KNIGHT: &str =
        "meditation/textures/polpo/videos/knight.webp";
    pub const VIDEO_MUKBANG: &str =
        "meditation/textures/polpo/videos/mukbang.webp";
    pub const VIDEO_PANDA: &str = "meditation/textures/polpo/videos/panda.webp";
    pub const VIDEO_PUPPY: &str = "meditation/textures/polpo/videos/puppy.webp";
    pub const VIDEO_SANDWICH: &str =
        "meditation/textures/polpo/videos/sandwich.webp";
    pub const VIDEO_VAMPIRE: &str =
        "meditation/textures/polpo/videos/vampire.webp";
}

pub mod portraits {
    use bevy::math::Vec2;

    pub const FOLDER: &str = "characters/portraits";

    pub const WINNIE: &str = "characters/portraits/winnie1.png";
    pub const PHOEBE: &str = "characters/portraits/princess1.png";
    pub const MARIE: &str = "characters/portraits/marie1.png";
    pub const BOLT: &str = "characters/portraits/bolt1.png";
    pub const CAPY: &str = "characters/portraits/capy1.png";
    pub const CAT: &str = "characters/portraits/cat1.png";
    pub const GINGER_CAT: &str = "characters/portraits/gingercat1.png";
    pub const WHITE_CAT: &str = "characters/portraits/whitecat1.png";
    pub const EMIL: &str = "characters/portraits/emil1.png";
    pub const MASTER: &str = "characters/portraits/master1.png";
    pub const COOPER: &str = "characters/portraits/cooper1.png";
    pub const REDHEAD: &str = "characters/portraits/redhead1.png";
    pub const SAMIZDAT: &str = "characters/portraits/samizdat1.png";
    pub const OTTER: &str = "characters/portraits/otter1.png";

    /// All portraits are the same size.
    pub const SIZE_PX: Vec2 = Vec2::splat(384.0);
}

pub mod character_atlases {
    pub const FOLDER: &str = "characters/atlases";

    pub const WINNIE: &str = "characters/atlases/winnie1.png";
    pub const MARIE: &str = "characters/atlases/marie1.png";
    pub const SAMIZDAT: &str = "characters/atlases/samizdat1.png";
    pub const BOLT: &str = "characters/atlases/bolt1.png";
    pub const WHITE_CAT: &str = "characters/atlases/whitecat1.png";
    pub const COOPER: &str = "characters/atlases/cooper1.png";
    pub const OTTER: &str = "characters/atlases/otter1.png";
    pub const PHOEBE: &str = "characters/atlases/phoebe1.png";

    pub const WINNIE_COLS: usize = 12;
}

pub mod misc {
    pub const LOADING_SCREEN_BUNNY_ATLAS: &str =
        "misc/loading_screens/bunny_atlas.png";
    pub const LOADING_SCREEN_SPACE_ATLAS: &str =
        "misc/loading_screens/space_atlas.png";
    pub const LOADING_SCREEN_WINNIE_IN_BATHROOM_ATLAS: &str =
        "misc/loading_screens/winnie_in_bathroom_atlas.png";
    pub const LOADING_SCREEN_HEDGEHOG_ATLAS: &str =
        "misc/loading_screens/hedgehog_atlas.png";
    pub const EMOJI_ATLAS: &str = "misc/emoji_atlas.png";
}

pub mod ui {
    use bevy::math::Vec2;

    pub const DIALOG_BOX: &str = "ui/dialog_box.png";
    pub const HEARTBEAT_ATLAS: &str = "ui/hud/heartbeat_atlas.png";
    pub const TIME_ATLAS: &str = "ui/hud/time_atlas.png";

    pub const HEARTBEAT_ATLAS_SIZE: Vec2 = Vec2::splat(70.0);
}
