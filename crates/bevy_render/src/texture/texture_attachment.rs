use super::CachedTexture;
use crate::{
    frame_graph::{
        PassBuilder, PassNodeBuilderExt, TextureViewEdge, TransientRenderPassColorAttachment,
        TransientRenderPassDepthStencilAttachment, TransientTextureView,
        TransientTextureViewDescriptor,
    },
    render_resource::{Texture, TextureFormat, TextureView},
};
use alloc::sync::Arc;
use bevy_color::LinearRgba;
use core::sync::atomic::{AtomicBool, Ordering};
use wgpu::{
    LoadOp, Operations, RenderPassColorAttachment, RenderPassDepthStencilAttachment, StoreOp,
};

/// A wrapper for a [`CachedTexture`] that is used as a [`RenderPassColorAttachment`].
#[derive(Clone)]
pub struct ColorAttachment {
    pub texture: CachedTexture,
    pub resolve_target: Option<CachedTexture>,
    pub previous_frame_texture: Option<CachedTexture>,
    clear_color: Option<LinearRgba>,
    is_first_call: Arc<AtomicBool>,
}

impl ColorAttachment {
    pub fn new(
        texture: CachedTexture,
        resolve_target: Option<CachedTexture>,
        previous_frame_texture: Option<CachedTexture>,
        clear_color: Option<LinearRgba>,
    ) -> Self {
        Self {
            texture,
            resolve_target,
            previous_frame_texture,
            clear_color,
            is_first_call: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn create_unsampled_transient_render_pass_color_attachment(
        &self,
        pass_builder: &mut PassBuilder,
    ) -> TransientRenderPassColorAttachment {
        let first_call = self.is_first_call.fetch_and(false, Ordering::SeqCst);
        let view_ref = pass_builder.read_material(&self.texture.texture);
        let view = TextureViewEdge::Read(TransientTextureView {
            texture: view_ref,
            desc: TransientTextureViewDescriptor::default(),
        });

        TransientRenderPassColorAttachment {
            view,
            depth_slice: None,
            resolve_target: None,
            ops: Operations {
                load: match (self.clear_color, first_call) {
                    (Some(clear_color), true) => LoadOp::Clear(clear_color.into()),
                    (None, _) | (Some(_), false) => LoadOp::Load,
                },
                store: StoreOp::Store,
            },
        }
    }

    pub fn create_transient_render_pass_color_attachment(
        &self,
        pass_builder: &mut PassBuilder,
    ) -> TransientRenderPassColorAttachment {
        if let Some(view) = self.resolve_target.as_ref() {
            let first_call = self.is_first_call.fetch_and(false, Ordering::SeqCst);

            let view_ref = pass_builder.read_material(&view.texture);
            let view = TextureViewEdge::Read(TransientTextureView {
                texture: view_ref,
                desc: TransientTextureViewDescriptor::default(),
            });

            let resolve_target_ref = pass_builder.read_material(&self.texture.texture);
            let resolve_target_view = TextureViewEdge::Read(TransientTextureView {
                texture: resolve_target_ref,
                desc: TransientTextureViewDescriptor::default(),
            });

            TransientRenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: Some(resolve_target_view),
                ops: Operations {
                    load: match (self.clear_color, first_call) {
                        (Some(clear_color), true) => LoadOp::Clear(clear_color.into()),
                        (None, _) | (Some(_), false) => LoadOp::Load,
                    },
                    store: StoreOp::Store,
                },
            }
        } else {
            self.create_unsampled_transient_render_pass_color_attachment(pass_builder)
        }
    }

    /// Get this texture view as an attachment. The attachment will be cleared with a value of
    /// `clear_color` if this is the first time calling this function, otherwise it will be loaded.
    ///
    /// The returned attachment will always have writing enabled (`store: StoreOp::Load`).
    pub fn get_attachment(&self) -> RenderPassColorAttachment<'_> {
        if let Some(resolve_target) = self.resolve_target.as_ref() {
            let first_call = self.is_first_call.fetch_and(false, Ordering::SeqCst);

            RenderPassColorAttachment {
                view: &resolve_target.default_view,
                depth_slice: None,
                resolve_target: Some(&self.texture.default_view),
                ops: Operations {
                    load: match (self.clear_color, first_call) {
                        (Some(clear_color), true) => LoadOp::Clear(clear_color.into()),
                        (None, _) | (Some(_), false) => LoadOp::Load,
                    },
                    store: StoreOp::Store,
                },
            }
        } else {
            self.get_unsampled_attachment()
        }
    }

    /// Get this texture view as an attachment, without the resolve target. The attachment will be cleared with
    /// a value of `clear_color` if this is the first time calling this function, otherwise it will be loaded.
    ///
    /// The returned attachment will always have writing enabled (`store: StoreOp::Load`).
    pub fn get_unsampled_attachment(&self) -> RenderPassColorAttachment<'_> {
        let first_call = self.is_first_call.fetch_and(false, Ordering::SeqCst);

        RenderPassColorAttachment {
            view: &self.texture.default_view,
            depth_slice: None,
            resolve_target: None,
            ops: Operations {
                load: match (self.clear_color, first_call) {
                    (Some(clear_color), true) => LoadOp::Clear(clear_color.into()),
                    (None, _) | (Some(_), false) => LoadOp::Load,
                },
                store: StoreOp::Store,
            },
        }
    }

    pub(crate) fn mark_as_cleared(&self) {
        self.is_first_call.store(false, Ordering::SeqCst);
    }
}

/// A wrapper for a [`TextureView`] that is used as a depth-only [`RenderPassDepthStencilAttachment`].
#[derive(Clone)]
pub struct DepthAttachment {
    pub view: TextureView,
    clear_value: Option<f32>,
    is_first_call: Arc<AtomicBool>,
}

impl DepthAttachment {
    pub fn new(view: TextureView, clear_value: Option<f32>) -> Self {
        Self {
            view,
            clear_value,
            is_first_call: Arc::new(AtomicBool::new(clear_value.is_some())),
        }
    }

    pub fn create_transient_render_pass_depth_stencil_attachment(
        &self,
        store: StoreOp,
        texture: &Texture,
        pass_builder: &mut PassBuilder,
    ) -> TransientRenderPassDepthStencilAttachment {
        let first_call = self
            .is_first_call
            .fetch_and(store != StoreOp::Store, Ordering::SeqCst);

        let view_ref = pass_builder.read_material(texture);
        let view = TextureViewEdge::Read(TransientTextureView {
            texture: view_ref,
            desc: TransientTextureViewDescriptor::default(),
        });

        TransientRenderPassDepthStencilAttachment {
            view,
            depth_ops: Some(Operations {
                load: if first_call {
                    // If first_call is true, then a clear value will always have been provided in the constructor
                    LoadOp::Clear(self.clear_value.unwrap())
                } else {
                    LoadOp::Load
                },
                store,
            }),
            stencil_ops: None,
        }
    }

    /// Get this texture view as an attachment. The attachment will be cleared with a value of
    /// `clear_value` if this is the first time calling this function with `store` == [`StoreOp::Store`],
    /// and a clear value was provided, otherwise it will be loaded.
    pub fn get_attachment(&self, store: StoreOp) -> RenderPassDepthStencilAttachment<'_> {
        let first_call = self
            .is_first_call
            .fetch_and(store != StoreOp::Store, Ordering::SeqCst);

        RenderPassDepthStencilAttachment {
            view: &self.view,
            depth_ops: Some(Operations {
                load: if first_call {
                    // If first_call is true, then a clear value will always have been provided in the constructor
                    LoadOp::Clear(self.clear_value.unwrap())
                } else {
                    LoadOp::Load
                },
                store,
            }),
            stencil_ops: None,
        }
    }
}

/// A wrapper for a [`TextureView`] that is used as a [`RenderPassColorAttachment`] for a view
/// target's final output texture.
#[derive(Clone)]
pub struct OutputColorAttachment {
    pub view: TextureView,
    pub view_format: TextureFormat,
    is_first_call: Arc<AtomicBool>,
}

impl OutputColorAttachment {
    pub fn new(view: TextureView, view_format: TextureFormat) -> Self {
        Self {
            view,
            view_format,
            is_first_call: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Get this texture view as an attachment. The attachment will be cleared with a value of
    /// the provided `clear_color` if this is the first time calling this function, otherwise it
    /// will be loaded.
    pub fn get_attachment(&self, clear_color: Option<LinearRgba>) -> RenderPassColorAttachment<'_> {
        let first_call = self.is_first_call.fetch_and(false, Ordering::SeqCst);

        RenderPassColorAttachment {
            view: &self.view,
            depth_slice: None,
            resolve_target: None,
            ops: Operations {
                load: match (clear_color, first_call) {
                    (Some(clear_color), true) => LoadOp::Clear(clear_color.into()),
                    (None, _) | (Some(_), false) => LoadOp::Load,
                },
                store: StoreOp::Store,
            },
        }
    }

    /// Returns `true` if this attachment has been written to by a render pass.
    // we re-use is_first_call atomic to track usage, which assumes that calls to get_attachment
    // are always consumed by a render pass that writes to the attachment
    pub fn needs_present(&self) -> bool {
        !self.is_first_call.load(Ordering::SeqCst)
    }
}
