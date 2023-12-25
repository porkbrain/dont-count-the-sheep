use std::marker::PhantomData;

use bevy::asset::load_internal_asset;
use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResourcePlugin;
use bevy::render::render_graph::{self, RenderGraph};
use bevy::render::render_resource::*;
use bevy::render::renderer::RenderContext;
use bevy::render::{Render, RenderApp, RenderSet};
use bevy::sprite::Material2dPlugin;
use bevy::window::{PrimaryWindow, WindowResized};

use self::pipeline::GiTargets;
use crate::gi::compositing::{
    setup_post_processing_camera, CameraTargets, PostProcessingMaterial,
};
use crate::gi::pipeline::{
    system_queue_bind_groups, system_setup_gi_pipeline, GiTargetsWrapper,
    LightPassPipeline, LightPassPipelineBindGroups,
};
use crate::gi::pipeline_assets::{
    system_extract_pipeline_assets, system_prepare_pipeline_assets,
    LightPassPipelineAssets,
};
use crate::gi::resource::ComputedTargetSizes;
use crate::prelude::BevyMagicLight2DSettings;

mod constants;
mod pipeline;
mod pipeline_assets;
mod types_gpu;

pub mod compositing;
pub mod resource;
pub mod types;
pub mod util;

const WORKGROUP_SIZE: u32 = 8;

pub trait LightScene:
    Component + TypePath + Send + Sync + Sized + Clone + Default + 'static
{
    /// Some unique number that we can use to generate handles IDs in increasing
    /// order.
    const HANDLE_START: u128 = 23475629871623176235;

    fn camera_order() -> isize;
    fn render_layer_index() -> u8;

    fn build(app: &mut App) {
        app
            .add_plugins(
                ExtractResourcePlugin::<GiTargetsWrapper<Self>>::default(),
            )
            .init_resource::<CameraTargets<Self>>()
            .init_resource::<GiTargetsWrapper<Self>>()
            .add_plugins(
                Material2dPlugin::<PostProcessingMaterial<Self>>::default(),
            )
            .add_systems(
                PreStartup,
                (setup_post_processing_camera::<Self>
                    .after(system_setup_gi_pipeline::<Self>),
                    system_setup_gi_pipeline::<Self>.after(detect_target_sizes)
                ),
            )
            .add_systems(PreUpdate, handle_window_resize::<Self>);

        load_internal_asset!(
            app,
            Self::shader_gi_camera(),
            "shaders/gi_camera.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            Self::shader_gi_types(),
            "shaders/gi_types.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            Self::shader_gi_attenuation(),
            "shaders/gi_attenuation.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            Self::shader_gi_halton(),
            "shaders/gi_halton.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            Self::shader_gi_math(),
            "shaders/gi_math.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            Self::shader_gi_raymarch(),
            "shaders/gi_raymarch.wgsl",
            Shader::from_wgsl
        );

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .add_systems(
                ExtractSchedule,
                system_extract_pipeline_assets::<Self>,
            )
            .add_systems(
                Render,
                (
                    system_prepare_pipeline_assets::<Self>
                        .in_set(RenderSet::Prepare),
                    system_queue_bind_groups::<Self>.in_set(RenderSet::Queue),
                ),
            );

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph
            .add_node("light_pass_2d", LightPass2DNode::<Self>::default());
        render_graph.add_node_edge(
            "light_pass_2d",
            bevy::render::main_graph::node::CAMERA_DRIVER,
        )
    }

    fn finish(app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<LightPassPipelineAssets<Self>>()
            .init_resource::<LightPassPipeline<Self>>();
    }

    fn post_processing_quad() -> Handle<Mesh> {
        Handle::weak_from_u128(Self::HANDLE_START + 1)
    }
    fn post_processing_material() -> Handle<PostProcessingMaterial<Self>> {
        Handle::weak_from_u128(Self::HANDLE_START + 2)
    }
    fn floor_image_handle() -> Handle<Image> {
        Handle::weak_from_u128(Self::HANDLE_START + 3)
    }
    fn sdf_target() -> Handle<Image> {
        Handle::weak_from_u128(Self::HANDLE_START + 4)
    }
    fn ss_probe_target() -> Handle<Image> {
        Handle::weak_from_u128(Self::HANDLE_START + 5)
    }
    fn ss_bounce_target() -> Handle<Image> {
        Handle::weak_from_u128(Self::HANDLE_START + 6)
    }
    fn ss_blend_target() -> Handle<Image> {
        Handle::weak_from_u128(Self::HANDLE_START + 7)
    }
    fn ss_filter_target() -> Handle<Image> {
        Handle::weak_from_u128(Self::HANDLE_START + 8)
    }
    fn ss_pose_target() -> Handle<Image> {
        Handle::weak_from_u128(Self::HANDLE_START + 9)
    }
    fn shader_gi_camera() -> Handle<Shader> {
        Handle::weak_from_u128(Self::HANDLE_START + 10)
    }
    fn shader_gi_types() -> Handle<Shader> {
        Handle::weak_from_u128(Self::HANDLE_START + 11)
    }
    fn shader_gi_attenuation() -> Handle<Shader> {
        Handle::weak_from_u128(Self::HANDLE_START + 12)
    }
    fn shader_gi_halton() -> Handle<Shader> {
        Handle::weak_from_u128(Self::HANDLE_START + 13)
    }
    fn shader_gi_math() -> Handle<Shader> {
        Handle::weak_from_u128(Self::HANDLE_START + 14)
    }
    fn shader_gi_raymarch() -> Handle<Shader> {
        Handle::weak_from_u128(Self::HANDLE_START + 15)
    }
}

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BevyMagicLight2DSettings>()
            .init_resource::<ComputedTargetSizes>()
            .add_systems(PreStartup, detect_target_sizes);
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<ComputedTargetSizes>();
    }
}

#[derive(Default)]
struct LightPass2DNode<T> {
    phantom: PhantomData<T>,
}

#[allow(clippy::too_many_arguments)]
pub fn handle_window_resize<T: LightScene>(
    mut assets_mesh: ResMut<Assets<Mesh>>,
    mut assets_material: ResMut<Assets<PostProcessingMaterial<T>>>,
    mut assets_image: ResMut<Assets<Image>>,

    query_window: Query<&Window, With<PrimaryWindow>>,

    res_plugin_config: Res<BevyMagicLight2DSettings>,
    mut res_target_sizes: ResMut<ComputedTargetSizes>,
    mut res_gi_targets_wrapper: ResMut<GiTargetsWrapper<T>>,
    mut res_camera_targets: ResMut<CameraTargets<T>>,

    mut window_resized_evr: EventReader<WindowResized>,
) {
    for _ in window_resized_evr.read() {
        let window = query_window
            .get_single()
            .expect("Expected exactly one primary window");

        *res_target_sizes = ComputedTargetSizes::from_window(
            window,
            &res_plugin_config.target_scaling_params,
        );

        assets_mesh.insert(
            T::post_processing_quad(),
            Mesh::from(shape::Quad::new(Vec2::new(
                res_target_sizes.primary_target_size.x,
                res_target_sizes.primary_target_size.y,
            ))),
        );

        assets_material.insert(
            T::post_processing_material(),
            PostProcessingMaterial::create(
                &res_camera_targets,
                &res_gi_targets_wrapper,
            ),
        );

        *res_gi_targets_wrapper = GiTargetsWrapper {
            targets: Some(GiTargets::create(
                &mut assets_image,
                &res_target_sizes,
            )),
        };
        *res_camera_targets =
            CameraTargets::create(&mut assets_image, &res_target_sizes);
    }
}

#[rustfmt::skip]
pub fn detect_target_sizes(
        query_window:      Query<&Window, With<PrimaryWindow>>,

        res_plugin_config: Res<BevyMagicLight2DSettings>,
    mut res_target_sizes:  ResMut<ComputedTargetSizes>,
)
{
    let window = query_window.get_single().expect("Expected exactly one primary window");
    *res_target_sizes = ComputedTargetSizes::from_window(window, &res_plugin_config.target_scaling_params);
}

impl<T: LightScene> render_graph::Node for LightPass2DNode<T> {
    fn update(&mut self, _world: &mut World) {}

    fn run(
        &self,
        _: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        if let Some(pipeline_bind_groups) =
            world.get_resource::<LightPassPipelineBindGroups<T>>()
        {
            let pipeline_cache = world.resource::<PipelineCache>();
            let pipeline = world.resource::<LightPassPipeline<T>>();
            let target_sizes = world.resource::<ComputedTargetSizes>();

            if let (
                Some(sdf_pipeline),
                Some(ss_probe_pipeline),
                Some(ss_bounce_pipeline),
                Some(ss_blend_pipeline),
                Some(ss_filter_pipeline),
            ) = (
                pipeline_cache.get_compute_pipeline(pipeline.sdf_pipeline),
                pipeline_cache.get_compute_pipeline(pipeline.ss_probe_pipeline),
                pipeline_cache
                    .get_compute_pipeline(pipeline.ss_bounce_pipeline),
                pipeline_cache.get_compute_pipeline(pipeline.ss_blend_pipeline),
                pipeline_cache
                    .get_compute_pipeline(pipeline.ss_filter_pipeline),
            ) {
                let sdf_w = target_sizes.sdf_target_usize.x;
                let sdf_h = target_sizes.sdf_target_usize.y;

                let mut pass = render_context
                    .command_encoder()
                    .begin_compute_pass(&ComputePassDescriptor {
                        label: Some("light_pass_2d"),
                    });

                {
                    let grid_w = sdf_w / WORKGROUP_SIZE;
                    let grid_h = sdf_h / WORKGROUP_SIZE;
                    pass.set_bind_group(
                        0,
                        &pipeline_bind_groups.sdf_bind_group,
                        &[],
                    );
                    pass.set_pipeline(sdf_pipeline);
                    pass.dispatch_workgroups(grid_w, grid_h, 1);
                }

                {
                    let grid_w =
                        target_sizes.probe_grid_usize.x / WORKGROUP_SIZE;
                    let grid_h =
                        target_sizes.probe_grid_usize.y / WORKGROUP_SIZE;
                    pass.set_bind_group(
                        0,
                        &pipeline_bind_groups.ss_probe_bind_group,
                        &[],
                    );
                    pass.set_pipeline(ss_probe_pipeline);
                    pass.dispatch_workgroups(grid_w, grid_h, 1);
                }

                {
                    let grid_w =
                        target_sizes.probe_grid_usize.x / WORKGROUP_SIZE;
                    let grid_h =
                        target_sizes.probe_grid_usize.y / WORKGROUP_SIZE;
                    pass.set_bind_group(
                        0,
                        &pipeline_bind_groups.ss_bounce_bind_group,
                        &[],
                    );
                    pass.set_pipeline(ss_bounce_pipeline);
                    pass.dispatch_workgroups(grid_w, grid_h, 1);
                }

                {
                    let grid_w =
                        target_sizes.probe_grid_usize.x / WORKGROUP_SIZE;
                    let grid_h =
                        target_sizes.probe_grid_usize.y / WORKGROUP_SIZE;
                    pass.set_bind_group(
                        0,
                        &pipeline_bind_groups.ss_blend_bind_group,
                        &[],
                    );
                    pass.set_pipeline(ss_blend_pipeline);
                    pass.dispatch_workgroups(grid_w, grid_h, 1);
                }

                {
                    let aligned = util::align_to_work_group_grid(
                        target_sizes.primary_target_isize,
                    )
                    .as_uvec2();
                    let grid_w = aligned.x / WORKGROUP_SIZE;
                    let grid_h = aligned.y / WORKGROUP_SIZE;
                    pass.set_bind_group(
                        0,
                        &pipeline_bind_groups.ss_filter_bind_group,
                        &[],
                    );
                    pass.set_pipeline(ss_filter_pipeline);
                    pass.dispatch_workgroups(grid_w, grid_h, 1);
                }
            }
        } else {
            log::warn!("Failed to get bind groups");
        }

        Ok(())
    }
}
