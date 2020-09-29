/***
TODO:
  attachment load/store config?
  begin_render_pass with some config, perhaps the load/store, or not. Which attachment is bound.
  depth only pass?

  some getters.. num_attachments(), get_attachment_format(idx) ?


***/

#[derive(Debug)]
struct ColorAttachment {
    data: ColorAttachmentData,
    color_format: wgpu::TextureFormat,
    assembled: Option<ColorAttachmentAssembled>,
    clear_color: [f64; 4],
}

#[derive(Debug)]
struct ColorAttachmentAssembled {
    multisample_texture: Option<wgpu::Texture>,
    attachment_view: Option<wgpu::TextureView>,
    resolve_view: Option<wgpu::TextureView>,
}
#[derive(Debug)]
enum ColorAttachmentData {
    Surface {
        surface: wgpu::Surface,
        swap_chain: Option<wgpu::SwapChain>,
        frame: Option<wgpu::SwapChainFrame>,
    },
    Texture {
        color_texture: Option<wgpu::Texture>,
    },
}
#[derive(Debug)]
pub struct Framebuffer {
    color_attachments: Vec<ColorAttachment>,
    sample_count: u32,
    resolution: (u32, u32),

    live_frame: Vec<wgpu::SwapChainFrame>,

    depth_stencil_format: Option<wgpu::TextureFormat>,
    depth_stencil_view: Option<wgpu::TextureView>,

    present_mode: wgpu::PresentMode,

    dirty: bool,
}

impl Framebuffer {
    pub fn new() -> Framebuffer {
        Framebuffer {
            color_attachments: Vec::new(),
            depth_stencil_view: None,
            live_frame: Vec::new(),
            sample_count: 1,
            resolution: (0, 0),
            depth_stencil_format: None,
            present_mode: wgpu::PresentMode::Mailbox,
            dirty: true,
        }
    }

    pub fn new_from_window<W: raw_window_handle::HasRawWindowHandle>(
        instance: &wgpu::Instance,
        window: &W,
color_format: wgpu::TextureFormat
    ) -> Framebuffer {
        let surface = unsafe { instance.create_surface(window) };
        Self::new_from_surface(surface, color_format)
    }
    pub fn new_from_surface(surface: wgpu::Surface, color_format: wgpu::TextureFormat) -> Framebuffer {
        let mut fb = Framebuffer::new();
        fb.add_surface_attachment(surface, color_format);
        fb
    }

    pub fn new_with_texture(color_format: wgpu::TextureFormat) -> Framebuffer {
        let mut fb = Framebuffer::new();
        fb.add_texture_attachment(color_format);
        fb
    }

    pub fn add_surface_attachment(&mut self, surface: wgpu::Surface,color_format: wgpu::TextureFormat) {
        self.color_attachments.push(ColorAttachment {
            data: ColorAttachmentData::Surface {
                surface,
                swap_chain: None,
                frame: None,
            },
            color_format: color_format,
            assembled: None,
            clear_color: [0.0, 0.0, 0.0, 0.0f64],
        });
    }

    pub fn add_texture_attachment(&mut self, color_format: wgpu::TextureFormat) {
        self.color_attachments.push(ColorAttachment {
            data: ColorAttachmentData::Texture {
                color_texture: None,
            },
            color_format,
            assembled: None,
            clear_color: [0.0, 0.0, 0.0, 0.0f64],
        });
    }

    pub fn set_clear_color(&mut self, clear_color: &[f64; 4]) {
        for attachment in &mut self.color_attachments {
            attachment.clear_color = *clear_color;
        }
    }

    pub fn set_sample_count(&mut self, sample_count: u32) {
        self.sample_count = sample_count;
        self.dirty = true;
    }

    pub fn set_depth_format(&mut self, format: wgpu::TextureFormat) {
        self.depth_stencil_format = Some(format);
    }

    // Returns sample count, 1 meaning no multisampling
    pub fn sample_count(&self) -> u32 {
        self.sample_count
    }

    // Returns width
    pub fn width(&self) -> u32 {
        self.resolution.0
    }

    // Returns height
    pub fn height(&self) -> u32 {
        self.resolution.1
    }

    // Sets the resolution for all the attachments.
    // Invalidates resources, requires `assemble`
    pub fn set_resolution(&mut self, width: u32, height: u32) {
        self.resolution = (width, height);
        self.dirty = true;
    }

    pub fn assemble(&mut self, device: &wgpu::Device) {
        debug_assert!(
            !self.needs_present(),
            "Live swapchain frames that were not presented while reconfiguring!"
        );
        if !self.dirty {
            // Nothing changed
            return;
        }
        self.dirty = false;

        let surface_colorformat = if cfg!(target_arch = "wasm32") {
            wgpu::TextureFormat::Bgra8Unorm
        } else {
            wgpu::TextureFormat::Bgra8UnormSrgb
        };

        for attachment in &mut self.color_attachments {
            let mut output_view = None;
            match attachment.data {
                ColorAttachmentData::Surface {
                    ref mut surface,
                    ref mut swap_chain,
                    ref mut frame,
                } => {
                    let sc_desc = wgpu::SwapChainDescriptor {
                        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                        format: attachment.color_format,
                        width: self.resolution.0,
                        height: self.resolution.1,
                        present_mode: self.present_mode,
                    };

                    *frame = None; // SwapChainFrame must be dropped debug creating creating new swapchain
                    let new_swap_chain = device.create_swap_chain(&surface, &sc_desc);
                    *swap_chain = Some(new_swap_chain);
                }
                ColorAttachmentData::Texture {
                    ref mut color_texture,
                } => {
                    let texture = device.create_texture(&wgpu::TextureDescriptor {
                        label: Some("Framebuffer Texture"),
                        size: wgpu::Extent3d {
                            width: self.resolution.0,
                            height: self.resolution.1,
                            depth: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: attachment.color_format,
                        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED,
                    });
                    let tex_view = texture.create_view(&wgpu::TextureViewDescriptor {
                        label: Some("Framebuffer Texture view"),
                        format: Some(attachment.color_format),
                        dimension: Some(wgpu::TextureViewDimension::D2),
                        aspect: wgpu::TextureAspect::All,
                        base_mip_level: 0,
                        level_count: None,
                        base_array_layer: 0,
                        array_layer_count: None,
                    });
                    output_view = Some(tex_view);
                    *color_texture = Some(texture);
                }
            }

            if self.sample_count > 1 {
                let msaa_texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("Framebuffer MSAA Texture"),
                    size: wgpu::Extent3d {
                        width: self.resolution.0,
                        height: self.resolution.1,
                        depth: 1,
                    },
                    mip_level_count: 1,
                    sample_count: self.sample_count,
                    dimension: wgpu::TextureDimension::D2,
                    format: surface_colorformat,
                    usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                });
                let msaa_view =
                    Some(msaa_texture.create_view(&wgpu::TextureViewDescriptor::default()));

                attachment.assembled = Some(ColorAttachmentAssembled {
                    multisample_texture: Some(msaa_texture),
                    attachment_view: msaa_view,
                    resolve_view: output_view,
                });
            } else {
                attachment.assembled = Some(ColorAttachmentAssembled {
                    multisample_texture: None,
                    attachment_view: output_view,
                    resolve_view: None,
                });
            }
        }

        if let Some(depth_format) = self.depth_stencil_format {
            self.depth_stencil_view = Some(
                device
                    .create_texture(&wgpu::TextureDescriptor {
                        size: wgpu::Extent3d {
                            width: self.width(),
                            height: self.height(),
                            depth: 1,
                        },
                        mip_level_count: 1,
                        sample_count: self.sample_count,
                        dimension: wgpu::TextureDimension::D2,
                        format: depth_format,
                        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                        label: Some("wgpu-util depth texture"),
                    })
                    .create_view(&wgpu::TextureViewDescriptor::default()),
            );
        }
    }

    // Check if there is a live swapchain frame
    pub fn needs_present(&self) -> bool {
        !self.live_frame.is_empty()
    }

    // If the Framebuffer has an live swapchain frame, present it.
    // Needs to be called between after the last render pass that uses it
    // is submitted (but before acquiring a new one)
    pub fn present(&mut self) {
        debug_assert!(self.needs_present());

        // wgpu currently present on Drop
        self.live_frame.clear();
    }

    pub fn begin_render_pass<'a>(
        &'a mut self,
        encoder: &'a mut wgpu::CommandEncoder,
    ) -> wgpu::RenderPass<'a> {
        // The lifetimes above are telling that the RenderPass must not be
        // dropped before self or encoder, as the RenderPass will refer to
        // values in them

        debug_assert!(self.live_frame.is_empty());
        assert!(!self.dirty, "Framebuffer was modified but not reconfigured");

        let mut color_attachments = Vec::new();

        // Start acquire the swapchain frames in separate loop,
        // so that we can mutate self to store them, when the
        // renderpass borrows it
        for attachment in &self.color_attachments {
            match &attachment.data {
                ColorAttachmentData::Surface { swap_chain, .. } => {
                    let swap_chain = swap_chain.as_ref().expect(
                    "swap chain is missing, did you remember to call `resize` before `begin_render_pass`",
                );
                    let new_frame = swap_chain
                        .get_current_frame()
                        .expect("Timeout when acquiring next swap chain texture");

                    self.live_frame.push(new_frame);
                }
                _ => {}
            }
        }

        let mut swapchain_idx = 0;
        // TODO: retain the vec, update only when dirty or surface attachment
        for attachment in &self.color_attachments {
            let attachment_view;
            let resolve_view;
            match &attachment.data {
                ColorAttachmentData::Surface { .. } => {
                    let frame_view = &self.live_frame.get(swapchain_idx).unwrap().output.view;

                    let assembled = attachment
                        .assembled
                        .as_ref()
                        .expect("Unconfigured attachment, did you call assemble()?");

                    if assembled.attachment_view.is_some() {
                        attachment_view = assembled.attachment_view.as_ref().unwrap();
                        resolve_view = Some(frame_view);
                    } else {
                        attachment_view = frame_view;
                        resolve_view = None;
                    }

                    swapchain_idx += 1;
                }
                ColorAttachmentData::Texture { color_texture: _ } => {
                    attachment_view = attachment
                        .assembled
                        .as_ref()
                        .unwrap()
                        .attachment_view
                        .as_ref()
                        .unwrap();
                    resolve_view = attachment.assembled.as_ref().unwrap().resolve_view.as_ref();
                }
            }

            color_attachments.push(wgpu::RenderPassColorAttachmentDescriptor {
                attachment: attachment_view,
                resolve_target: resolve_view,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: attachment.clear_color[0],
                        g: attachment.clear_color[1],
                        b: attachment.clear_color[2],
                        a: attachment.clear_color[3],
                    }),
                    store: true,
                },
            });
        }

        let depth_stencil_attachment = self.depth_stencil_view.as_ref().map(|tex| {
            wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: tex,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: false,
                }),
                stencil_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0),
                    store: false,
                }),
            }
        });

        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &color_attachments,
            depth_stencil_attachment: depth_stencil_attachment,
        })
    }
}
