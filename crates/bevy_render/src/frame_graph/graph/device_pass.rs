use crate::frame_graph::{
    FrameGraph, FrameGraphContext, IndexHandle, Pass, PassNode, ResourceRelease, ResourceRequese,
};

#[derive(Default)]
pub struct DevicePass {
    pub pass: Option<Pass>,
    pub resource_release_array: Vec<ResourceRelease>,
    pub resource_request_array: Vec<ResourceRequese>,
    pub name: String,
}

impl DevicePass {
    pub fn request_resources(&self, ctx: &mut FrameGraphContext) {
        for resource in self.resource_request_array.iter() {
            ctx.resource_table.request_resource(
                resource,
                &ctx.render_device,
                ctx.transient_resource_cache,
            );
        }
    }

    pub fn release_resources(&self, ctx: &mut FrameGraphContext) {
        for handle in self.resource_release_array.iter() {
            ctx.resource_table
                .release_resource(handle, ctx.transient_resource_cache);
        }
    }

    pub fn execute(&self, ctx: &mut FrameGraphContext) {
        self.request_resources(ctx);

        if let Some(pass) = &self.pass {
            pass.render(
                &ctx.render_device,
                &mut ctx.command_buffers,
                &ctx.resource_table,
            );
        }
        self.release_resources(ctx);
    }

    pub fn extra(&mut self, graph: &mut FrameGraph, handle: &IndexHandle<PassNode>) {
        let pass_node = graph.get_pass_node(handle);

        let resource_request_array = pass_node
            .resource_request_array
            .iter()
            .map(|handle| graph.get_resource_node(handle).request())
            .collect();

        let resource_release_array = pass_node
            .resource_release_array
            .iter()
            .map(|handle| graph.get_resource_node(handle).release())
            .collect();

        let pass_node = graph.get_pass_node_mut(handle);

        let pass = pass_node.pass.take();

        self.resource_request_array = resource_request_array;
        self.pass = pass;
        self.resource_release_array = resource_release_array;

        self.name = pass_node.name.clone();
    }
}
