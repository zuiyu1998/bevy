use crate::core_2d::Transparent2d;
use bevy_ecs::prelude::*;
use bevy_log::error;
#[cfg(feature = "trace")]
use bevy_log::info_span;
use bevy_render::{
    camera::ExtractedCamera,
    diagnostic::RecordDiagnostics,
    render_phase::{TrackedRenderPass, ViewSortedRenderPhases},
    render_resource::StoreOp,
    renderer::{FrameGraphs, RenderContext, ViewQuery},
    view::{ExtractedView, ViewDepthTexture, ViewTarget},
};

pub fn main_transparent_pass_2d(
    world: &World,
    view: ViewQuery<(
        &ExtractedCamera,
        &ExtractedView,
        &ViewTarget,
        &ViewDepthTexture,
    )>,
    transparent_phases: Res<ViewSortedRenderPhases<Transparent2d>>,
    mut frame_graphs: ResMut<FrameGraphs>,
    ctx: RenderContext,
) {
    let view_entity = view.entity();
    let (camera, extracted_view, target, depth) = view.into_inner();

    let Some(transparent_phase) = transparent_phases.get(&extracted_view.retained_view_entity)
    else {
        return;
    };

    #[cfg(feature = "trace")]
    let _span = info_span!("main_transparent_pass_2d").entered();

    let diagnostics = ctx.diagnostic_recorder();
    let diagnostics = diagnostics.as_deref();

    let frame_graph = frame_graphs.get_or_insert(view_entity);
    let mut pass_builder = frame_graph.create_pass_builder("main_transparent_pass_2d_node");

    let color_attachment = target.create_transient_render_pass_color_attachment(&mut pass_builder);
    // NOTE: For the transparent pass we load the depth buffer. There should be no
    // need to write to it, but store is set to `true` as a workaround for issue #3776,
    // https://github.com/bevyengine/bevy/issues/3776
    // so that wgpu does not clear the depth buffer.
    // As the opaque and alpha mask passes run first, opaque meshes can occlude
    // transparent ones.
    let depth_stencil_attachment = depth
        .create_transient_render_pass_depth_stencil_attachment(StoreOp::Store, &mut pass_builder);

    {
        let mut render_pass_builder =
            pass_builder.create_render_pass_builder("main_transparent_pass_2d");

        render_pass_builder.add_color_attachment(color_attachment);
        render_pass_builder.set_depth_stencil_attachment(depth_stencil_attachment);

        let mut render_pass = TrackedRenderPass::new(ctx.render_device(), render_pass_builder);

        let pass_span = diagnostics.pass_span(&mut render_pass, "main_transparent_pass_2d");

        if let Some(viewport) = camera.viewport.as_ref() {
            render_pass.set_camera_viewport(viewport);
        }

        if !transparent_phase.items.is_empty() {
            #[cfg(feature = "trace")]
            let _transparent_span = info_span!("transparent_main_pass_2d").entered();
            if let Err(err) = transparent_phase.render(&mut render_pass, world, view_entity) {
                error!("Error encountered while rendering the transparent 2D phase {err:?}");
            }
        }

        pass_span.end(&mut render_pass);
    }

    // WebGL2 quirk: if ending with a render pass with a custom viewport, the viewport isn't
    // reset for the next render pass so add an empty render pass without a custom viewport
    #[cfg(all(feature = "webgl", target_arch = "wasm32", not(feature = "webgpu")))]
    if camera.viewport.is_some() {
        #[cfg(feature = "trace")]
        let _reset_viewport_pass_2d = info_span!("reset_viewport_pass_2d").entered();

        let mut render_pass_builder =
            pass_builder.create_render_pass_builder("reset_viewport_pass_2d");
        render_pass_builder.add_color_attachment(color_attachment);
    }
}
