//! All the assets that the game loads are stored as string paths here.

pub mod fonts {
    pub const FOLDER: &str = "fonts";

    pub const PIXEL1: &str = "fonts/shaperka.ttf";
    pub const PENCIL1: &str = "fonts/pencil.ttf";
}

pub mod apartment {
    pub const FOLDER: &str = "apartment";
    pub const MAP: &str = "apartment/map.ron";
    pub const BG: &str = "apartment/bg.png";

    pub const CLOUD_ATLAS: &str = "apartment/cloud_atlas.png";

    pub const BEDROOM_FURNITURE1: &str = "apartment/bedroom_furniture1.png";
    pub const BEDROOM_FURNITURE2: &str = "apartment/bedroom_furniture2.png";
    pub const BEDROOM_FURNITURE3: &str = "apartment/bedroom_furniture3.png";
    pub const BEDROOM_MAIN_DOOR: &str = "apartment/brown_door_atlas.png";

    pub const KITCHEN_FURNITURE1: &str = "apartment/kitchen_furniture1.png";
    pub const KITCHEN_FURNITURE2: &str = "apartment/kitchen_furniture2.png";
    pub const KITCHEN_FURNITURE3: &str = "apartment/kitchen_furniture3.png";

    pub const WINNIE_ATLAS: &str = "apartment/winnie_atlas.png";

    pub const WINNIE_SLEEPING: &str = "apartment/sleeping.png";
    pub const WINNIE_MEDITATING: &str = "apartment/meditating.png";

    pub const HALLWAY: &str = "apartment/hallway.png";
    pub const HALLWAY_DOORS: &str = "apartment/hallway_doors.png";
    pub const ELEVATOR_ATLAS: &str = "apartment/elevator_atlas.png";
}

pub mod downtown {
    pub const FOLDER: &str = "downtown";
    pub const BG: &str = "downtown/bg.png";
    pub const MAP: &str = "downtown/map.ron";
}

pub mod meditation {
    pub const FOLDER: &str = "meditation";

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

pub mod dialog {
    pub const FOLDER: &str = "dialog";

    pub const DIALOG_BUBBLE: &str = "dialog/bubble.png";
    pub const DIALOG_CHOICE: &str = "dialog/choice.png";
    pub const DIALOG_CHOICE_HIGHLIGHTED: &str = "dialog/choice_highlighted.png";
}

pub mod portraits {
    pub const FOLDER: &str = "characters/portraits";

    pub const WINNIE: &str = "characters/portraits/winnie1.png";
    pub const PHOEBE: &str = "characters/portraits/princess1.png";
    pub const MARIE: &str = "characters/portraits/widow1.png";
    pub const BOLT: &str = "characters/portraits/bolt1.png";
    pub const CAPY: &str = "characters/portraits/capy1.png";
    pub const CAT: &str = "characters/portraits/cat1.png";
    pub const EMIL: &str = "characters/portraits/emil1.png";
    pub const MASTER: &str = "characters/portraits/master1.png";
    pub const POOPER: &str = "characters/portraits/pooper1.png";
    pub const REDHEAD: &str = "characters/portraits/redhead1.png";
    pub const UNNAMED: &str = "characters/portraits/unnamed1.png";
    pub const OTTER: &str = "characters/portraits/otter1.png";
}

pub mod character_atlases {
    pub const FOLDER: &str = "characters/atlases";

    pub const WINNIE: &str = "characters/atlases/winnie1.png";
}
