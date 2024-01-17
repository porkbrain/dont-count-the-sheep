use std::marker::PhantomData;

use bevy::{
    prelude::*,
    reflect::TypePath,
    render::{
        mesh::MeshVertexBufferLayout,
        render_resource::{
            AsBindGroup, Extent3d, RenderPipelineDescriptor, ShaderDefVal,
            ShaderRef, SpecializedMeshPipelineError, TextureDescriptor,
            TextureDimension, TextureFormat, TextureUsages,
        },
        texture::BevyDefault,
        view::RenderLayers,
    },
    sprite::{Material2d, Material2dKey, MaterialMesh2dBundle},
};

use super::LightScene;
use crate::gi::{pipeline::GiTargetsWrapper, resource::ComputedTargetSizes};

/// Taken from `bevy_pbr`.
const MAX_CASCADES_PER_LIGHT: usize = 4;
/// Taken from `bevy_pbr`.
const MAX_DIRECTIONAL_LIGHTS: usize = 10;

#[derive(Component)]
pub struct PostProcessingQuad;

#[derive(AsBindGroup, Clone, TypePath, Asset)]
pub struct PostProcessingMaterial<T: LightScene> {
    #[texture(0)]
    #[sampler(1)]
    floor_image: Handle<Image>,

    #[texture(6)]
    #[sampler(7)]
    irradiance_image: Handle<Image>,

    phantom: PhantomData<T>,
}

impl<T: LightScene> PostProcessingMaterial<T> {
    pub fn create(
        camera_targets: &CameraTargets<T>,
        gi_targets_wrapper: &GiTargetsWrapper<T>,
    ) -> Self {
        Self {
            floor_image: camera_targets.floor_target.clone(),
            irradiance_image: gi_targets_wrapper
                .targets
                .as_ref()
                .expect("Targets must be initialized")
                .ss_filter_target
                .clone(),
            phantom: PhantomData,
        }
    }
}

#[derive(Resource, Default)]
pub struct CameraTargets<T> {
    pub floor_target: Handle<Image>,

    phantom: PhantomData<T>,
}

impl<T: LightScene> CameraTargets<T> {
    pub fn create(
        images: &mut Assets<Image>,
        sizes: &ComputedTargetSizes,
    ) -> Self {
        let target_size = Extent3d {
            width: sizes.primary_target_usize.x,
            height: sizes.primary_target_usize.y,
            ..default()
        };

        let mut floor_image = Image {
            texture_descriptor: TextureDescriptor {
                label: Some("target_floor"),
                size: target_size,
                dimension: TextureDimension::D2,
                format: TextureFormat::bevy_default(),
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
            ..default()
        };

        // Fill images data with zeroes.
        floor_image.resize(target_size);

        let floor_image_handle: Handle<Image> = T::floor_image_handle();

        images.insert(floor_image_handle.clone(), floor_image);

        Self {
            floor_target: floor_image_handle,
            phantom: PhantomData,
        }
    }
}

impl<T: LightScene> Material2d for PostProcessingMaterial<T> {
    fn fragment_shader() -> ShaderRef {
        "shaders/gi_post_processing.wgsl".into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let shader_defs = &mut descriptor
            .fragment
            .as_mut()
            .expect("Fragment shader empty")
            .shader_defs;
        shader_defs.push(ShaderDefVal::UInt(
            "MAX_DIRECTIONAL_LIGHTS".to_string(),
            MAX_DIRECTIONAL_LIGHTS as u32,
        ));
        shader_defs.push(ShaderDefVal::UInt(
            "MAX_CASCADES_PER_LIGHT".to_string(),
            MAX_CASCADES_PER_LIGHT as u32,
        ));
        Ok(())
    }
}

pub fn setup_post_processing_quad<T: LightScene>(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<PostProcessingMaterial<T>>>,
    mut images: ResMut<Assets<Image>>,
    mut camera_targets: ResMut<CameraTargets<T>>,

    target_sizes: Res<ComputedTargetSizes>,
    gi_targets_wrapper: Res<GiTargetsWrapper<T>>,
) {
    let quad = Mesh::from(shape::Quad::new(Vec2::new(
        target_sizes.primary_target_size.x,
        target_sizes.primary_target_size.y,
    )));

    meshes.insert(T::post_processing_quad(), quad);

    *camera_targets = CameraTargets::create(&mut images, &target_sizes);

    let material =
        PostProcessingMaterial::create(&camera_targets, &gi_targets_wrapper);
    materials.insert(T::post_processing_material(), material);

    // This specifies the layer used for the post processing camera, which
    // will be attached to the post processing camera and 2d quad.
    let layer = RenderLayers::layer(T::render_layer_index());

    cmd.spawn((
        T::default(),
        PostProcessingQuad,
        MaterialMesh2dBundle {
            mesh: T::post_processing_quad().into(),
            material: T::post_processing_material(),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 1.5),
                ..default()
            },
            ..default()
        },
        layer,
    ));
}
