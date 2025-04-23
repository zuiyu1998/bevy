use super::{
    FrameGraph, PassData, PassNode, RenderContext, ResourceTable, TransientResourceCache,
    TypeHandle, VirtualResource,
};
use crate::{renderer::RenderDevice, Result};

#[derive(Default)]
pub struct LogicPass {
    pub pass_data: Option<Box<dyn PassData>>,
    pub resource_release_array: Vec<TypeHandle<VirtualResource>>,
    pub resource_request_array: Vec<VirtualResource>,
    pub name: String,
}

impl LogicPass {
    pub fn request_resources(
        &self,
        device: &RenderDevice,
        transient_resource_cache: &mut TransientResourceCache,
        resource_table: &mut ResourceTable,
    ) {
        for resource in self.resource_request_array.iter() {
            resource_table.request_resource(resource, device, transient_resource_cache);
        }
    }

    pub fn release_resources(
        &self,
        transient_resource_cache: &mut TransientResourceCache,
        resource_table: &mut ResourceTable,
    ) {
        for handle in self.resource_release_array.iter() {
            resource_table.release_resource(handle, transient_resource_cache);
        }
    }
}

#[derive(Default)]
pub struct DevicePass {
    pub logic_passes: Vec<LogicPass>,
}

impl DevicePass {
    pub fn execute(&mut self, render_context: &mut RenderContext) -> Result<()> {
        self.begin(render_context);

        for logic_pass in self.logic_passes.iter() {
            if let Some(pass_data) = &logic_pass.pass_data {
                pass_data.execute(render_context)?;
            }
        }

        self.end(render_context);

        Ok(())
    }

    pub fn begin(&self, render_context: &mut RenderContext) {
        for logic_pass in self.logic_passes.iter() {
            logic_pass.request_resources(
                render_context.device,
                render_context.transient_resource_cache,
                &mut render_context.resource_table,
            );
        }
    }

    pub fn end(&self, render_context: &mut RenderContext) {
        for logic_pass in self.logic_passes.iter() {
            logic_pass.release_resources(
                render_context.transient_resource_cache,
                &mut render_context.resource_table,
            )
        }
    }

    pub fn extra(&mut self, graph: &mut FrameGraph, handle: TypeHandle<PassNode>) {
        let pass_node = &mut graph.pass_nodes[handle.index];
        let pass_data = pass_node.pass_data.take();

        let resource_request_array = pass_node
            .resource_request_array
            .iter()
            .map(|handle| graph.resources[handle.index].clone())
            .collect();

        let resource_release_array = pass_node.resource_release_array.clone();

        let logic_pass = LogicPass {
            pass_data,
            resource_request_array,
            resource_release_array,
            name: pass_node.name.clone(),
        };

        self.logic_passes.push(logic_pass);
    }
}
