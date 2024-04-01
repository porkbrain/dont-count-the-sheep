//! Emoji's are used to express emotions in a visual way.

use bevy::{
    math::vec2,
    prelude::*,
    utils::{Duration, Instant},
};
use common_assets::paths::EMOJI_ATLAS;
use common_visuals::{
    AtlasAnimation, AtlasAnimationEnd, AtlasAnimationStep, AtlasAnimationTimer,
};

use crate::Character;

/// Don't replace the current emoji too early, would look weird.
/// Since this is just a visual cue to the player, ignoring is not
/// a big deal.
///
/// If I wanted to be fancy I'd have queued up emojis.
/// But I fancy finishing this game.
const MIN_EMOJI_DURATION: Duration = Duration::from_millis(1000);

/// How large is a single emoji atlas tile.
const EMOJI_SIZE: Vec2 = vec2(24.0, 22.0);

/// Send this event to display an emoji.
///
/// This event might end up being ignored if
/// - the same emoji is already displayed
/// - the current emoji has not been displayed for long enough
#[derive(Event)]
pub struct DisplayEmojiEvent {
    /// Which emoji to display.
    pub emoji: EmojiKind,
    /// Each entity can only display one emoji at a time.
    /// The emoji will insert itself as a child of this entity unless it
    /// already exists.
    /// Then it will just update the existing emoji.
    pub on_parent: Entity,
    /// Who is the parent entity?
    /// The entity character mustn't change while the emoji is displayed.
    pub offset_for: Character,
}

/// Emojis represent moods or situations that are nice to visually convey to
/// the player.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EmojiKind {
    /// Short emoji animation
    Tired,
}

enum EmojiFrames {
    #[allow(dead_code)]
    Empty = 0,

    Tired1 = 1,
    Tired2 = 2,
    Tired3 = 3,
}

#[derive(Component)]
struct Emoji {
    kind: EmojiKind,
    started_at: Instant,
}

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DisplayEmojiEvent>().add_systems(
            Update,
            play_next.run_if(on_event::<DisplayEmojiEvent>()),
        );
    }
}

fn play_next(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut events: EventReader<DisplayEmojiEvent>,

    mut existing_emoji: Query<(Entity, &Parent, &mut Emoji, &mut TextureAtlas)>,
) {
    for event in events.read() {
        // this search is O(n) but there never are many emojis
        let existing_emoji = existing_emoji
            .iter_mut()
            .find(|(_, parent, ..)| parent.get() == event.on_parent);

        let entity = if let Some((entity, _, mut emoji, mut atlas)) =
            existing_emoji
        {
            if emoji.kind == event.emoji
                || emoji.started_at.elapsed() < MIN_EMOJI_DURATION
            {
                // let the current emoji play out
                continue;
            }

            // set new emoji
            *emoji = Emoji {
                kind: event.emoji,
                started_at: Instant::now(),
            };
            atlas.index = event.emoji.initial_frame();

            entity
        } else {
            let entity = cmd
                .spawn(Name::new("Emoji"))
                .insert(Emoji {
                    kind: event.emoji,
                    started_at: Instant::now(),
                })
                .insert(SpriteBundle {
                    texture: asset_server.load(EMOJI_ATLAS),
                    transform: Transform::from_translation(
                        // the z-index is a dirty hack to make sure the emoji
                        // is always in front of the character
                        event.offset_for.emoji_offset().extend(11.0),
                    ),
                    ..default()
                })
                .insert(TextureAtlas {
                    layout: layouts.add(TextureAtlasLayout::from_grid(
                        EMOJI_SIZE,
                        4,
                        1,
                        Some(Vec2::splat(2.0)),
                        Some(Vec2::splat(1.0)),
                    )),
                    index: event.emoji.initial_frame(),
                })
                .id();
            cmd.entity(event.on_parent).add_child(entity);

            entity
        };

        if let Some(first) = event.emoji.animation_first_frame() {
            cmd.entity(entity)
                .insert(AtlasAnimation {
                    first,
                    last: event.emoji.animation_last_frame(),
                    play: AtlasAnimationStep::Forward,
                    on_last_frame: AtlasAnimationEnd::DespawnItself,
                    extra_steps: event.emoji.extra_steps(),
                })
                .insert(AtlasAnimationTimer::new_fps(event.emoji.fps()));
        }
    }
}

impl Character {
    fn emoji_offset(self) -> Vec2 {
        let (size, ..) = self.sprite_atlas().unwrap_or_default();

        vec2(0.0, size.y + EMOJI_SIZE.y / 2.0)
    }
}

impl EmojiKind {
    fn initial_frame(self) -> usize {
        match self {
            Self::Tired => EmojiFrames::Tired1 as usize,
        }
    }

    /// Only [`Some`] if an animation.
    fn animation_first_frame(self) -> Option<usize> {
        match self {
            Self::Tired => Some(EmojiFrames::Tired2 as usize),
        }
    }

    fn animation_last_frame(self) -> usize {
        match self {
            Self::Tired => EmojiFrames::Tired3 as usize,
        }
    }

    fn fps(self) -> f32 {
        match self {
            Self::Tired => 3.0,
        }
    }

    fn extra_steps(self) -> Vec<AtlasAnimationStep> {
        match self {
            Self::Tired => vec![
                AtlasAnimationStep::Backward,
                AtlasAnimationStep::Forward,
                AtlasAnimationStep::Backward,
                AtlasAnimationStep::Forward,
                AtlasAnimationStep::Backward,
            ],
        }
    }
}
