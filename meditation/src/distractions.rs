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
pub struct WebPAnimationBundle {
    pub animation: Handle<WebPAnimation>,
    pub sprite: Sprite,
    pub target: Handle<Image>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

#[derive(Asset, TypePath, Debug, Clone)]
pub struct WebPAnimation {
    next_frame: Receiver<Image>,
    label: String,
}

pub struct WebPAnimationPlugin;

impl Plugin for WebPAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<WebPAnimationLoader>()
            .init_asset::<WebPAnimation>()
            // .register_asset_reflect::<WebpAnimation>()
            .add_systems(PostUpdate, load_next_frame);
    }

    fn finish(&self, _app: &mut App) {
        //
    }
}

pub fn load_next_frame(
    mut query: Query<(&mut Handle<Image>, &mut Handle<WebPAnimation>)>,
    animations: Res<Assets<WebPAnimation>>,
    mut images: ResMut<Assets<Image>>,
) {
    for (mut handle, receiver) in query.iter_mut() {
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
    }
}

pub fn xd(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands.spawn(WebPAnimationBundle {
        animation: asset_server.load("textures/distractions/test.webp"),
        ..default()
    });
}

#[derive(Serialize, Deserialize, Default, Debug)]

pub struct WebPAnimationLoader;

#[derive(Serialize, Deserialize, Default, Debug)]

pub struct WebPAnimationLoaderSettings;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum WebPAnimationLoaderError {
    #[error("Image loading error: {0}")]
    ImageError(#[from] image::ImageError),
    #[error("Could load shader: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not load texture file: {0}")]
    FileTexture(#[from] TextureError),
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
                // TODO: improve the frame processing
                thread::spawn(move || {
                    // TODO: one shot channel for confirmation of the first
                    // frame otherwise error this fn
                    if let Err(e) = foo(animation_frame, bytes.as_slice()) {
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

fn foo(
    animation_frame: Sender<Image>,
    bytes: &[u8],
) -> Result<(), WebPAnimationLoaderError> {
    loop {
        let decoder = WebPDecoder::new(bytes)?;
        let frames = decoder.into_frames().collect_frames()?;

        for frame in frames {
            let (width, height) = frame.buffer().dimensions();
            trace!("Creating image {width}x{height}");
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

            trace!("Sending image to channel");
            animation_frame.send(image).ok(); // animation no longer required
        }
    }
}
