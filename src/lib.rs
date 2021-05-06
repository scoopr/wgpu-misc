#[cfg(feature = "async")]
mod async_block;

#[cfg(feature = "framebuffer")]
mod framebuffer;

/// Re-export the dependent wgpu version, for easily using the same version
pub use wgpu;

#[cfg(feature = "async")]
pub use async_block::block_on;

#[cfg(feature = "framebuffer")]
pub use framebuffer::Framebuffer;
