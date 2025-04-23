use bevy_ecs::prelude::*;

use crate::{
    render_resource::PipelineCache,
    renderer::{RenderDevice, RenderQueue},
};

use super::{FrameGraph, RenderContext, SetupGraph, SetupGraphRunner};

pub fn setup_frame_graph_system(world: &mut World) {
    world.resource_scope(|world, mut graph: Mut<SetupGraph>| {
        graph.update(world);
    });
    let graph = world.resource::<SetupGraph>();

    let mut frame_graph = FrameGraph::default();

    if let Err(e) = SetupGraphRunner::run(graph, &mut frame_graph, world) {
        panic!("setup frame graph error: {}", e);
    }

    world.insert_resource(frame_graph);
}

pub fn compile_frame_graph_system(mut frame_graph: ResMut<FrameGraph>) {
    frame_graph.compile();
}

pub fn execute_frame_graph_system(
    mut frame_graph: ResMut<FrameGraph>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    pipeline_cache: Res<PipelineCache>,
) {
    let mut render_context = RenderContext::new(&render_device, &pipeline_cache);

    frame_graph.execute(&mut render_context);

    let mut command_buffers = vec![];

    for command_buffer in render_context.finish().into_iter() {
        let command_buffer = command_buffer.command_buffer();

        if command_buffer.is_some() {
            command_buffers.push(command_buffer.unwrap());
        }
    }

    render_queue.submit(command_buffers);
}
