//! Emoji's are used to express emotions in a visual way.

use bevy::{
    math::vec2,
    prelude::*,
    utils::{Duration, Instant},
};
use common_assets::paths::EMOJI_ATLAS;

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

        if let Some((entity, _, mut emoji, mut atlas)) = existing_emoji {
            if emoji.kind == event.emoji
                || emoji.started_at.elapsed() < MIN_EMOJI_DURATION
            {
                // let the current emoji play out
                return;
            }

            // set new emoji
            *emoji = Emoji {
                kind: event.emoji,
                started_at: Instant::now(),
            };

            if event.emoji.is_animation() {
                atlas.index = EmojiFrames::Empty as usize;
                // TODO: schedule next_emoji_kind animation
            } else {
                atlas.index = EmojiFrames::from(event.emoji) as usize;
            }
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
                    index: if event.emoji.is_animation() {
                        EmojiFrames::Empty as usize
                    } else {
                        EmojiFrames::from(event.emoji) as usize
                    },
                })
                .id();
            cmd.entity(event.on_parent).add_child(entity);

            if event.emoji.is_animation() {
                // TODO: schedule next_emoji_kind animation
            }
        }
    }
}

impl Character {
    fn emoji_offset(self) -> Vec2 {
        let (size, ..) = self.sprite_atlas().unwrap_or_default();

        vec2(0.0, size.y + EMOJI_SIZE.y / 2.0)
    }
}

impl From<EmojiKind> for EmojiFrames {
    fn from(emoji: EmojiKind) -> Self {
        match emoji {
            EmojiKind::Tired => EmojiFrames::Tired1,
        }
    }
}

impl EmojiKind {
    fn is_animation(self) -> bool {
        match self {
            Self::Tired => false,
        }
    }
}
