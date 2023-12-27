use bevy::{asset::Handle, render::render_resource::Shader};

pub const GI_SCREEN_PROBE_SIZE: i32 = 8;

pub const SHADER_GI_CAMERA: Handle<Shader> =
    Handle::weak_from_u128(61377524477160370295);

pub const SHADER_GI_TYPES: Handle<Shader> =
    Handle::weak_from_u128(61377524477160370295 + 1);

pub const SHADER_GI_ATTENUATION: Handle<Shader> =
    Handle::weak_from_u128(61377524477160370295 + 2);

pub const SHADER_GI_HALTON: Handle<Shader> =
    Handle::weak_from_u128(61377524477160370295 + 3);

pub const SHADER_GI_MATH: Handle<Shader> =
    Handle::weak_from_u128(61377524477160370295 + 4);

pub const SHADER_GI_RAYMARCH: Handle<Shader> =
    Handle::weak_from_u128(61377524477160370295 + 5);
