use bevy::prelude::*;
use crossbeam_channel::TryRecvError;

use crate::{FrameRate, WebpAnimation};

/// Runs all animation.
pub(crate) fn load_next_frame(
    mut query: Query<(
        &mut FrameRate,
        &mut Handle<Image>,
        &mut Handle<WebpAnimation>,
    )>,
    animations: Res<Assets<WebpAnimation>>,
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
