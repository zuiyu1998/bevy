use core::ops::Range;

use super::{RenderPassCommand, RenderPassContext};

pub struct DrawIndexedParameter {
    indices: Range<u32>,
    base_vertex: i32,
    instances: Range<u32>,
}

impl RenderPassCommand for DrawIndexedParameter {
    fn execute(&self, ctx: &mut RenderPassContext) {
        ctx.render_pass.draw_indexed(
            self.indices.clone(),
            self.base_vertex,
            self.instances.clone(),
        );
    }
}

pub struct DrawParameter {
    vertices: Range<u32>,
    instances: Range<u32>,
}

impl RenderPassCommand for DrawParameter {
    fn execute(&self, ctx: &mut RenderPassContext) {
        ctx.render_pass
            .draw(self.vertices.clone(), self.instances.clone());
    }
}
