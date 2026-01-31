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
    pub fn request_resources(&self, context: &mut FrameGraphContext) {
        for resource in self.resource_request_array.iter() {
            context.resource_table.request_resource(
                resource,
                &context.render_device,
                context.transient_resource_cache,
            );
        }
    }

    pub fn release_resources(&self, context: &mut FrameGraphContext) {
        for handle in self.resource_release_array.iter() {
            context
                .resource_table
                .release_resource(handle, context.transient_resource_cache);
        }
    }

    pub fn execute(&self, context: &mut FrameGraphContext) {
        self.request_resources(context);

        if let Some(pass) = &self.pass {
            pass.render(
                &mut context.command_buffers,
                &context.render_device,
                &context.resource_table,
                &context.pipeline_container,
            );
        }
        self.release_resources(context);
    }

    pub fn extra(&mut self, graph: &mut FrameGraph, index: IndexHandle<PassNode>) {
        let pass_node = graph.get_pass_node(&index);

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

        let pass_node = graph.get_pass_node_mut(&index);

        let pass = pass_node.pass.take();

        self.resource_request_array = resource_request_array;
        self.pass = pass;
        self.resource_release_array = resource_release_array;

        self.name = pass_node.name.clone();
    }
}
