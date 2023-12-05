use std::thread;

use crate::prelude::*;
use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    render::{
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        texture::TextureError,
    },
};
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use image::{codecs::webp::WebPDecoder, AnimationDecoder};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Bundle, Default)]
pub(crate) struct WebPAnimationBundle {
    pub(crate) frame_rate: WebPAnimationFrameRate,
    pub(crate) animation: Handle<WebPAnimation>,
    pub(crate) sprite: Sprite,
    pub(crate) target: Handle<Image>,
    pub(crate) transform: Transform,
    pub(crate) global_transform: GlobalTransform,
    pub(crate) visibility: Visibility,
    pub(crate) inherited_visibility: InheritedVisibility,
    pub(crate) view_visibility: ViewVisibility,
}

#[derive(Component)]
pub(crate) struct WebPAnimationFrameRate {
    /// How many ticks to wait before advancing to the next frame.
    pub(crate) ticks_per_frame: u32,
    pub(crate) current_tick: u32,
}

#[derive(Asset, TypePath, Debug, Clone)]
pub(crate) struct WebPAnimation {
    next_frame: Receiver<Image>,
    label: String,
}

#[derive(Serialize, Deserialize, Default, Debug)]

pub(crate) struct WebPAnimationLoader;

#[derive(Serialize, Deserialize, Default, Debug)]

pub(crate) struct WebPAnimationLoaderSettings;

pub(crate) struct WebPAnimationPlugin;

#[non_exhaustive]
#[derive(Debug, Error)]
pub(crate) enum WebPAnimationLoaderError {
    #[error("Image loading error: {0}")]
    ImageError(#[from] image::ImageError),
    #[error("Could load shader: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not load texture file: {0}")]
    FileTexture(#[from] TextureError),
}

impl Plugin for WebPAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<WebPAnimationLoader>()
            .init_asset::<WebPAnimation>()
            .add_systems(FixedUpdate, load_next_frame);
    }

    fn finish(&self, _app: &mut App) {
        //
    }
}

pub(crate) fn load_next_frame(
    mut query: Query<(
        &mut WebPAnimationFrameRate,
        &mut Handle<Image>,
        &mut Handle<WebPAnimation>,
    )>,
    animations: Res<Assets<WebPAnimation>>,
    mut images: ResMut<Assets<Image>>,
) {
    for (mut frame_rate, mut handle, receiver) in query.iter_mut() {
        frame_rate.current_tick += 1;

        if frame_rate.current_tick < frame_rate.ticks_per_frame {
            continue;
        }

        if let Some(animation) = animations.get(receiver.id()) {
            match animation.next_frame.try_recv() {
                Ok(next_frame) => {
                    *handle = images.add(next_frame);
                }
                Err(TryRecvError::Empty) => {
                    warn!("{}: frame skipped", animation.label);
                }
                Err(TryRecvError::Disconnected) => {
                    error!(
                        "{}: animation channel disconnected",
                        animation.label
                    );
                }
            }
        } else {
            warn!("{}: animation not found", receiver.id());
        }

        frame_rate.current_tick = 0;
    }
}

impl AssetLoader for WebPAnimationLoader {
    type Asset = WebPAnimation;
    type Settings = WebPAnimationLoaderSettings;
    type Error = WebPAnimationLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a WebPAnimationLoaderSettings,
        load_context: &'a mut LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<WebPAnimation, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            let (animation_frame, next_frame) = crossbeam_channel::bounded(60); // TODO
            let label = load_context.path().display().to_string();

            {
                let label = label.clone();
                // TODO: multiple animations per thread
                thread::spawn(move || {
                    // TODO: one shot channel for confirmation of the first
                    // frame otherwise error this fn
                    if let Err(e) =
                        spawn_decoder_thread(animation_frame, bytes.as_slice())
                    {
                        error!("{label}: frame processing error: {e}");
                    }
                });
            }

            trace!("Spawned channel for WebP animation from {label}");

            Ok(Self::Asset { next_frame, label })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["webp"]
    }
}

fn spawn_decoder_thread(
    animation_frame: Sender<Image>,
    bytes: &[u8],
) -> Result<(), WebPAnimationLoaderError> {
    loop {
        let frames = WebPDecoder::new(bytes)?.into_frames();
        for frame in frames {
            match frame {
                Ok(frame) => {
                    let (width, height) = frame.buffer().dimensions();
                    let image = Image::new(
                        Extent3d {
                            width,
                            height,
                            ..default()
                        },
                        TextureDimension::D2,
                        frame.into_buffer().into_raw(),
                        TextureFormat::Rgba8Unorm,
                    );
                    // animation no longer required
                    animation_frame.send(image).ok();
                }
                Err(e) => {
                    error!("Frame decoding error: {e}");
                }
            }
        }
    }
}

impl Default for WebPAnimationFrameRate {
    fn default() -> Self {
        Self {
            ticks_per_frame: 1,
            current_tick: 0,
        }
    }
}

impl WebPAnimationFrameRate {
    pub(crate) fn new(ticks_per_frame: u32) -> Self {
        Self {
            ticks_per_frame,
            current_tick: 0,
        }
    }
}
