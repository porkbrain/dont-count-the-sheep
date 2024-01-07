pub mod apartment {
    pub const BG: &str = "apartment/bg.png";

    pub const BEDROOM_FURNITURE1: &str = "apartment/bedroom_furniture1.png";
    pub const BEDROOM_FURNITURE2: &str = "apartment/bedroom_furniture2.png";
    pub const BEDROOM_FURNITURE3: &str = "apartment/bedroom_furniture3.png";

    pub const KITCHEN_FURNITURE1: &str = "apartment/kitchen_furniture1.png";
    pub const KITCHEN_FURNITURE2: &str = "apartment/kitchen_furniture2.png";
    pub const KITCHEN_FURNITURE3: &str = "apartment/kitchen_furniture3.png";

    pub const WINNIE_ATLAS: &str = "apartment/winnie_atlas.png";

    pub const WINNIE_SLEEPING: &str = "apartment/sleeping.png";
    pub const WINNIE_MEDITATING: &str = "apartment/meditating.png";

    pub const APARTMENT_MAP: &str = "apartment/map.ron";
}

pub mod meditation {
    pub const LOADING_SCREEN: &str = "meditation/textures/loading_screen.png";

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
