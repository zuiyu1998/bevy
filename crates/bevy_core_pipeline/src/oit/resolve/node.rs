use bevy_camera::{MainPassResolutionOverride, Viewport};
use bevy_ecs::prelude::*;
use bevy_render::{
    camera::ExtractedCamera,
    diagnostic::RecordDiagnostics,
    frame_graph::TransientBindGroupHandle,
    render_phase::TrackedRenderPass,
    render_resource::PipelineCache,
    renderer::{FrameGraphs, RenderContext, ViewQuery},
    view::{ViewDepthTexture, ViewTarget, ViewUniformOffset},
};

use crate::prepass::DepthPrepass;

use super::{OitResolveBindGroup, OitResolvePipeline, OitResolvePipelineId};

pub fn oit_resolve(
    view: ViewQuery<(
        &ExtractedCamera,
        &ViewTarget,
        &ViewUniformOffset,
        &OitResolvePipelineId,
        &ViewDepthTexture,
        &OitResolveBindGroup,
        Option<&MainPassResolutionOverride>,
        Has<DepthPrepass>,
    )>,
    resolve_pipeline: Option<Res<OitResolvePipeline>>,
    pipeline_cache: Res<PipelineCache>,
    ctx: RenderContext,
    mut frame_graphs: ResMut<FrameGraphs>,
) {
    let view_entity = view.entity();

    let (
        camera,
        view_target,
        view_uniform,
        oit_resolve_pipeline_id,
        depth,
        bind_group,
        resolution_override,
        depth_prepass,
    ) = view.into_inner();

    // This *must* run after main_transparent_pass_3d to reset the `oit_atomic_counter` and `oit_heads` buffer
    // Otherwise transparent pass will construct a corrupted linked list(can have circular references which causes infinite loop and device lost) on the next pass.
    let Some(resolve_pipeline) = resolve_pipeline else {
        return;
    };

    let Some(pipeline) = pipeline_cache.get_render_pipeline(oit_resolve_pipeline_id.0) else {
        return;
    };

    let diagnostics = ctx.diagnostic_recorder();
    let diagnostics = diagnostics.as_deref();

    let frame_graph = frame_graphs.get_or_insert(view_entity);

    let depth_bind_group = if !depth_prepass {
        let depth_texture_view = depth.get_texture_view_handle(frame_graph);

        let bind_group = TransientBindGroupHandle::build(
            &pipeline_cache.get_bind_group_layout(&resolve_pipeline.oit_depth_bind_group_layout),
        )
        .set_label("oit_resolve_depth_bind_group")
        .push(depth_texture_view)
        .finished();

        Some(bind_group)
    } else {
        None
    };

    let mut pass_builder = frame_graph.create_pass_builder("oit_resolve_node");

    let color_attachment =
        view_target.create_transient_render_pass_color_attachment(&mut pass_builder);
    let mut render_pass_builder = pass_builder.create_render_pass_builder("oit_resolve");

    render_pass_builder.add_color_attachment(color_attachment);

    let mut render_pass = TrackedRenderPass::new(ctx.render_device(), render_pass_builder);

    let pass_span = diagnostics.pass_span(&mut render_pass, "oit_resolve");

    if let Some(viewport) =
        Viewport::from_viewport_and_override(camera.viewport.as_ref(), resolution_override)
    {
        render_pass.set_camera_viewport(&viewport);
    }

    render_pass.set_render_pipeline(pipeline);
    render_pass.set_bind_group_handle(0, &bind_group, &[view_uniform.offset]);
    if let Some(depth_bind_group) = &depth_bind_group {
        render_pass.set_bind_group_handle(1, depth_bind_group, &[]);
    }
    render_pass.draw(0..3, 0..1);

    pass_span.end(&mut render_pass);
}
