use bevy_render::{
    render_graph::{CommandQueue, Node, ResourceSlots, SystemNode},
    render_resource::{BufferInfo, BufferUsage, RenderResourceBinding, RenderResourceBindings},
    renderer::{RenderContext, RenderResourceContext},
};

use crate::{
    light::{Light, LightRaw},
    uniform,
};
use bevy_core::bytes::{AsBytes, Byteable};
use bevy_transform::prelude::*;
use legion::prelude::*;

#[derive(Default)]
pub struct LightsNode {
    command_queue: CommandQueue,
    max_lights: usize,
}

impl LightsNode {
    pub fn new(max_lights: usize) -> Self {
        LightsNode {
            max_lights,
            command_queue: CommandQueue::default(),
        }
    }
}

impl Node for LightsNode {
    fn update(
        &mut self,
        _world: &World,
        _resources: &Resources,
        render_context: &mut dyn RenderContext,
        _input: &ResourceSlots,
        _output: &mut ResourceSlots,
    ) {
        self.command_queue.execute(render_context);
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct LightCount {
    pub num_lights: [u32; 4],
}

unsafe impl Byteable for LightCount {}

impl SystemNode for LightsNode {
    fn get_system(&self) -> Box<dyn Schedulable> {
        let mut light_buffer = None;
        let mut staging_buffer = None;
        let mut lights_are_dirty = true;
        // TODO: merge these
        let mut tmp_count_buffer = None;
        let mut tmp_light_buffer = None;
        let mut command_queue = self.command_queue.clone();
        let max_lights = self.max_lights;
        (move |render_resource_context: Res<Box<dyn RenderResourceContext>>,
               // TODO: this write on RenderResourceAssignments will prevent this system from running in parallel with other systems that do the same
               mut render_resource_bindings: ResMut<RenderResourceBindings>,
               world: &mut SubWorld,
               query: &mut Query<(Read<Light>, Read<Transform>, Read<Translation>)>| {
                let render_resource_context = &**render_resource_context;
            
                let light_count = query.iter().iter().count();
                let size = std::mem::size_of::<LightRaw>();
                let light_count_size = std::mem::size_of::<LightCount>();
                let light_array_size = size * light_count;
                let light_array_max_size = size * state.max_lights;
                let current_light_uniform_size = light_count_size + light_array_size;
                let max_light_uniform_size = light_count_size + light_array_max_size;
            
                if let Some(staging_buffer) = state.staging_buffer {
                    if light_count == 0 {
                        return;
                    }
            
                    render_resource_context.map_buffer(staging_buffer);
                } else {
                    let buffer = render_resource_context.create_buffer(BufferInfo {
                        size: max_light_uniform_size,
                        buffer_usage: BufferUsage::UNIFORM | BufferUsage::COPY_SRC | BufferUsage::COPY_DST,
                        ..Default::default()
                    });
                    render_resource_bindings.set(
                        uniform::LIGHTS,
                        RenderResourceBinding::Buffer {
                            buffer,
                            range: 0..max_light_uniform_size as u64,
                            dynamic_index: None,
                        },
                    );
                    light_buffer = Some(buffer);
            
                    let staging_buffer = render_resource_context.create_buffer(BufferInfo {
                        size: max_light_uniform_size,
                        buffer_usage: BufferUsage::COPY_SRC | BufferUsage::MAP_WRITE,
                        mapped_at_creation: true,
                    });
                    staging_buffer = Some(staging_buffer);
                }
            
                let staging_buffer = staging_buffer.unwrap();
                render_resource_context.write_mapped_buffer(
                    staging_buffer,
                    0..current_light_uniform_size as u64,
                    &mut |data, _renderer| {
                        // light count
                        data[0..light_count_size].copy_from_slice([light_count as u32, 0, 0, 0].as_bytes());
            
                        // light array
                        for ((light, transform, translation), slot) in
                            query.iter().iter().zip(data[light_count_size..current_light_uniform_size].chunks_exact_mut(size))
                        {
                            slot.copy_from_slice(
                                LightRaw::from(&light, &transform.value, &translation).as_bytes(),
                            );
                        }
                    },
                );
                render_resource_context.unmap_buffer(staging_buffer);
                let light_buffer = light_buffer.unwrap();
                command_queue.copy_buffer_to_buffer(
                    staging_buffer,
                    0,
                    light_buffer,
                    0,
                    max_light_uniform_size as u64,
                );
        })
        .system()
    }
}