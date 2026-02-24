use crate::{
    core_3d::Transparent3d,
    oit::{resolve::OitResolvePipelineId, OrderIndependentTransparencySettings},
};
use bevy_camera::{MainPassResolutionOverride, Viewport};
use bevy_ecs::prelude::*;
use bevy_log::error;
#[cfg(feature = "trace")]
use bevy_log::info_span;
use bevy_render::{
    camera::ExtractedCamera,
    diagnostic::RecordDiagnostics,
    render_phase::{TrackedRenderPass, ViewSortedRenderPhases},
    render_resource::{PipelineCache, StoreOp},
    renderer::{FrameGraphs, RenderContext, ViewQuery},
    view::{ExtractedView, ViewDepthTexture, ViewTarget},
};

pub fn main_transparent_pass_3d(
    world: &World,
    view: ViewQuery<(
        &ExtractedCamera,
        &ExtractedView,
        &ViewTarget,
        &ViewDepthTexture,
        Option<&MainPassResolutionOverride>,
        Has<OrderIndependentTransparencySettings>,
        Option<&OitResolvePipelineId>,
    )>,
    transparent_phases: Res<ViewSortedRenderPhases<Transparent3d>>,
    mut frame_graphs: ResMut<FrameGraphs>,
    ctx: RenderContext,
) {
    let view_entity = view.entity();

    let (
        camera,
        extracted_view,
        target,
        depth,
        resolution_override,
        has_oit,
        oit_resolve_pipeline_id,
    ) = view.into_inner();

    let Some(transparent_phase) = transparent_phases.get(&extracted_view.retained_view_entity)
    else {
        return;
    };

    let frame_graph = frame_graphs.get_or_insert(view_entity);
    let mut pass_builder = frame_graph.create_pass_builder("main_transparent_pass_3d_node");

    if !transparent_phase.items.is_empty() {
        #[cfg(feature = "trace")]
        let _main_transparent_pass_3d_span = info_span!("main_transparent_pass_3d").entered();

        let diagnostics = ctx.diagnostic_recorder();
        let diagnostics = diagnostics.as_deref();

        if has_oit {
            // We can't run transparent phase if OitResolvePipelineId is not ready
            // Otherwise we will write to `oit_atomic_counter` and `oit_heads` buffer without resetting them
            // which causes corrupted linked list(can have circular references) on the next pass
            let Some(oit_resolve_pipeline_id) = oit_resolve_pipeline_id else {
                return;
            };
            let pipeline_cache = world.resource::<PipelineCache>();
            if pipeline_cache
                .get_render_pipeline(oit_resolve_pipeline_id.0)
                .is_none()
            {
                return;
            }
        }

        let color_attachment =
            target.create_transient_render_pass_color_attachment(&mut pass_builder);
        let depth_stencil_attachment = depth.create_transient_render_pass_depth_stencil_attachment(
            StoreOp::Store,
            &mut pass_builder,
        );
        let mut render_pass_builder =
            pass_builder.create_render_pass_builder("main_transparent_pass_3d");

        render_pass_builder
            .add_color_attachment(color_attachment)
            .set_depth_stencil_attachment(depth_stencil_attachment);

        let mut render_pass = TrackedRenderPass::new(ctx.render_device(), render_pass_builder);

        let pass_span = diagnostics.pass_span(&mut render_pass, "main_transparent_pass_3d");

        if let Some(viewport) =
            Viewport::from_viewport_and_override(camera.viewport.as_ref(), resolution_override)
        {
            render_pass.set_camera_viewport(&viewport);
        }

        if let Err(err) = transparent_phase.render(&mut render_pass, world, view_entity) {
            error!("Error encountered while rendering the transparent phase {err:?}");
        }

        pass_span.end(&mut render_pass);
    }

    // WebGL2 quirk: if ending with a render pass with a custom viewport, the viewport isn't
    // reset for the next render pass so add an empty render pass without a custom viewport
    #[cfg(all(feature = "webgl", target_arch = "wasm32", not(feature = "webgpu")))]
    if camera.viewport.is_some() {
        #[cfg(feature = "trace")]
        let _reset_viewport_pass_3d = info_span!("reset_viewport_pass_3d").entered();

        let color_attachment =
            target.create_transient_render_pass_color_attachment(&mut pass_builder);
        let mut render_pass_builder =
            pass_builder.create_render_pass_builder("reset_viewport_pass_3d");

        render_pass_builder.add_color_attachment(color_attachment);
    }
}
