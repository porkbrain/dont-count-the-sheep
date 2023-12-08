//! Knows about loading .webp files.
//! We assume the files are animations.
//! The files operated on by the [`image`] crate.

use std::thread;

use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
    render::{
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        texture::TextureError,
    },
};
use crossbeam_channel::Sender;
use image::{codecs::webp::WebPDecoder, AnimationDecoder};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize, Deserialize, Default, Debug)]

pub struct WebpLoader;

#[derive(Serialize, Deserialize, Default, Debug)]

pub struct LoaderSettings;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum LoaderError {
    #[error("Image loading error: {0}")]
    ImageError(#[from] image::ImageError),
    #[error("Could load shader: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not load texture file: {0}")]
    FileTexture(#[from] TextureError),
}

impl AssetLoader for WebpLoader {
    type Asset = super::WebpAnimation;
    type Settings = LoaderSettings;
    type Error = LoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a LoaderSettings,
        load_context: &'a mut LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
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
) -> Result<(), LoaderError> {
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
