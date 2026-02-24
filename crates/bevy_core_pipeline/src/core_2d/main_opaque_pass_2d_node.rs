use crate::core_2d::Opaque2d;
use bevy_ecs::prelude::*;
use bevy_log::error;
#[cfg(feature = "trace")]
use bevy_log::info_span;
use bevy_render::{
    camera::ExtractedCamera,
    diagnostic::RecordDiagnostics,
    render_phase::{TrackedRenderPass, ViewBinnedRenderPhases},
    render_resource::StoreOp,
    renderer::{FrameGraphs, RenderContext, ViewQuery},
    view::{ExtractedView, ViewDepthTexture, ViewTarget},
};

use super::AlphaMask2d;

pub fn main_opaque_pass_2d(
    world: &World,
    view: ViewQuery<(
        &ExtractedCamera,
        &ExtractedView,
        &ViewTarget,
        &ViewDepthTexture,
    )>,
    opaque_phases: Res<ViewBinnedRenderPhases<Opaque2d>>,
    alpha_mask_phases: Res<ViewBinnedRenderPhases<AlphaMask2d>>,
    mut frame_graphs: ResMut<FrameGraphs>,
    ctx: RenderContext,
) {
    let view_entity = view.entity();
    let (camera, extracted_view, target, depth) = view.into_inner();

    let (Some(opaque_phase), Some(alpha_mask_phase)) = (
        opaque_phases.get(&extracted_view.retained_view_entity),
        alpha_mask_phases.get(&extracted_view.retained_view_entity),
    ) else {
        return;
    };

    if opaque_phase.is_empty() && alpha_mask_phase.is_empty() {
        return;
    }

    #[cfg(feature = "trace")]
    let _span = info_span!("main_opaque_pass_2d").entered();

    let diagnostics = ctx.diagnostic_recorder();
    let diagnostics = diagnostics.as_deref();

    let frame_graph = frame_graphs.get_or_insert(view_entity);

    let mut pass_builder = frame_graph.create_pass_builder("main_opaque_pass_2d_node");

    let color_attachment = target.create_transient_render_pass_color_attachment(&mut pass_builder);
    let depth_stencil_attachment = depth
        .create_transient_render_pass_depth_stencil_attachment(StoreOp::Store, &mut pass_builder);

    let mut render_pass_builder = pass_builder.create_render_pass_builder("main_opaque_pass_2d");

    render_pass_builder.add_color_attachment(color_attachment);
    render_pass_builder.set_depth_stencil_attachment(depth_stencil_attachment);

    let mut render_pass = TrackedRenderPass::new(ctx.render_device(), render_pass_builder);

    let pass_span = diagnostics.pass_span(&mut render_pass, "main_opaque_pass_2d");

    if let Some(viewport) = camera.viewport.as_ref() {
        render_pass.set_camera_viewport(viewport);
    }

    if !opaque_phase.is_empty() {
        #[cfg(feature = "trace")]
        let _opaque_span = info_span!("opaque_main_pass_2d").entered();
        if let Err(err) = opaque_phase.render(&mut render_pass, world, view_entity) {
            error!("Error encountered while rendering the 2d opaque phase {err:?}");
        }
    }

    if !alpha_mask_phase.is_empty() {
        #[cfg(feature = "trace")]
        let _alpha_mask_span = info_span!("alpha_mask_main_pass_2d").entered();
        if let Err(err) = alpha_mask_phase.render(&mut render_pass, world, view_entity) {
            error!("Error encountered while rendering the 2d alpha mask phase {err:?}");
        }
    }

    pass_span.end(&mut render_pass);
}
