#[derive(Debug)]
enum FramebufferTarget {
    Surface {
        surface: wgpu::Surface,
        swap_chain: Option<wgpu::SwapChain>,
        frame: Option<wgpu::SwapChainFrame>,
    },
    Texture {
        color_format: wgpu::TextureFormat,
        color_texture: Option<wgpu::Texture>,
        texture_view: Option<wgpu::TextureView>,
    },
}
#[derive(Debug)]
pub struct Framebuffer {
    target: FramebufferTarget,
    // surface: wgpu::Surface,
    // swap_chain: Option<wgpu::SwapChain>,
    depth_texture_view: Option<wgpu::TextureView>,
    clear_color: [f64; 4],
    //    resolution: (u32, u32),
    configuration: FramebufferConfiguration,
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct FramebufferConfiguration {
    pub resolution: (u32, u32),
    pub depth_format: Option<wgpu::TextureFormat>,
}

impl FramebufferConfiguration {
    pub fn with_resolution(&self, width: u32, height: u32) -> FramebufferConfiguration {
        let mut ret = self.clone();
        ret.resolution = (width, height);
        ret
    }
    pub fn with_depth_format(&self, depth_format: wgpu::TextureFormat) -> FramebufferConfiguration {
        let mut ret = self.clone();
        ret.depth_format = Some(depth_format);
        ret
    }
    pub fn with_no_depth(&self) -> FramebufferConfiguration {
        let mut ret = self.clone();
        ret.depth_format = None;
        ret
    }
    pub fn width(&self) -> u32 {
        self.resolution.0
    }
    pub fn height(&self) -> u32 {
        self.resolution.1
    }
}

impl Framebuffer {
    pub fn new_from_window<W: raw_window_handle::HasRawWindowHandle>(
        instance: &wgpu::Instance,
        window: &W,
    ) -> Framebuffer {
        let surface = unsafe { instance.create_surface(window) };
        Self::new_from_surface(surface)
    }
    pub fn new_from_surface(surface: wgpu::Surface) -> Framebuffer {
        Framebuffer {
            target: FramebufferTarget::Surface {
                surface,
                swap_chain: None,
                frame: None,
            },
            depth_texture_view: None,
            clear_color: [0f64, 0f64, 0f64, 1f64],
            configuration: FramebufferConfiguration::default(), //            resolution: (0, 0),
        }
    }

    pub fn new_texture(color_format: wgpu::TextureFormat) -> Framebuffer {
        Framebuffer {
            target: FramebufferTarget::Texture {
                color_format,
                color_texture: None,
                texture_view: None,
            },
            depth_texture_view: None,
            clear_color: [0f64, 0f64, 0f64, 1f64],
            configuration: FramebufferConfiguration::default(),
        }
    }

    pub fn configuration(&self) -> &FramebufferConfiguration {
        &self.configuration
    }

    pub fn set_clear_color(&mut self, clear_color: &[f64; 4]) {
        self.clear_color = *clear_color;
    }

    pub fn reconfigure(
        &mut self,
        device: &wgpu::Device,
        new_configuration: &FramebufferConfiguration,
    ) {
        if new_configuration == &self.configuration {
            // Nothing changed
            return;
        }

        self.configuration = new_configuration.clone(); //resolution = (width, height);
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            // TODO: Allow srgb unconditionally
            format: if cfg!(target_arch = "wasm32") {
                wgpu::TextureFormat::Bgra8Unorm
            } else {
                wgpu::TextureFormat::Bgra8UnormSrgb
            },
            width: self.configuration.width(),
            height: self.configuration.height(),
            present_mode: wgpu::PresentMode::Mailbox,
        };
        match &mut self.target {
            FramebufferTarget::Surface {
                ref mut surface,
                ref mut swap_chain,
                ref mut frame,
            } => {
                *frame = None; // SwapChainFrame must be dropped debug creating creating new swapchain
                let new_swap_chain = device.create_swap_chain(&surface, &sc_desc);
                *swap_chain = Some(new_swap_chain);
            }
            FramebufferTarget::Texture {
                ref color_format,
                ref mut color_texture,
                ref mut texture_view,
            } => {
                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("Framebuffer Texture"),
                    size: wgpu::Extent3d {
                        width: self.configuration.width(),
                        height: self.configuration.height(),
                        depth: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: *color_format,
                    usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED,
                });
                let tex_view = texture.create_view(&wgpu::TextureViewDescriptor {
                    label: Some("Framebuffer Texture view"),
                    format: Some(*color_format),
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    aspect: wgpu::TextureAspect::All,
                    base_mip_level: 0,
                    level_count: None,
                    base_array_layer: 0,
                    array_layer_count: None,
                });
                *texture_view = Some(tex_view);
                *color_texture = Some(texture);
            }
        }

        if let Some(depth_format) = self.configuration.depth_format {
            self.depth_texture_view = Some(
                device
                    .create_texture(&wgpu::TextureDescriptor {
                        size: wgpu::Extent3d {
                            width: self.configuration.width(),
                            height: self.configuration.height(),
                            depth: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: depth_format,
                        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                        label: Some("wgpu-util depth texture"),
                    })
                    .create_view(&wgpu::TextureViewDescriptor::default()),
            );
        }
    }

    pub fn color_format(&self) -> Option<wgpu::TextureFormat> {
        // TODO:
        Some(wgpu::TextureFormat::Bgra8UnormSrgb)
        // Some(wgpu::TextureFormat::Bgra8Unorm)
    }
    pub fn sample_count(&self) -> u32 {
        1 // TODO:
    }

    pub fn begin_render_pass<'a>(
        &'a mut self,
        encoder: &'a mut wgpu::CommandEncoder,
    ) -> wgpu::RenderPass<'a> {
        // The lifetimes above are telling that the RenderPass must not be
        // dropped before self or encoder, as the RenderPass will refer to
        // values in them

        match &mut self.target {
            FramebufferTarget::Surface {
                surface: _,
                swap_chain,
                ref mut frame,
            } => {
                let swap_chain = swap_chain.as_mut().expect(
                    "swap chain is missing, did you remember to call `resize` before `begin_render_pass`",
                );
                *frame = None;
                let new_frame = swap_chain
                    .get_current_frame()
                    .expect("Timeout when acquiring next swap chain texture");

                *frame = Some(new_frame);
                let frame_view = &frame.as_mut().unwrap().output.view;

                let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: frame_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: self.clear_color[0],
                        g: self.clear_color[1],
                        b: self.clear_color[2],
                        a: self.clear_color[3],
                    }),
                    store: true
                }
                // load_op: wgpu::LoadOp::Clear,
                // store_op: wgpu::StoreOp::Store,
                // clear_color: wgpu::Color {
                //     r: self.clear_color[0],
                //     g: self.clear_color[1],
                //     b: self.clear_color[2],
                //     a: self.clear_color[3],
                // },
            }],
                    depth_stencil_attachment: self.depth_texture_view.as_ref().map(|tex| {
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
                            // depth_load_op: wgpu::LoadOp::Clear,
                            // depth_store_op: wgpu::StoreOp::Store,
                            // stencil_load_op: wgpu::LoadOp::Clear,
                            // stencil_store_op: wgpu::StoreOp::Store,
                            // clear_depth: 1.0,
                            // clear_stencil: 0,
                            // depth_read_only: false,
                            // stencil_read_only: false,
                        }
                    }),
                });
                pass
            }
            FramebufferTarget::Texture {
                color_texture: _,
                color_format: _,
                texture_view,
            } => {
                let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &texture_view.as_ref().expect("Texture not configured"),
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: self.clear_color[0],
                                g: self.clear_color[1],
                                b: self.clear_color[2],
                                a: self.clear_color[3],
                            }),
                            store: true,
                        },
                    }],
                    depth_stencil_attachment: self.depth_texture_view.as_ref().map(|tex| {
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
                            // depth_load_op: wgpu::LoadOp::Clear,
                            // depth_store_op: wgpu::StoreOp::Store,
                            // stencil_load_op: wgpu::LoadOp::Clear,
                            // stencil_store_op: wgpu::StoreOp::Store,
                            // clear_depth: 1.0,
                            // clear_stencil: 0,
                            // depth_read_only: false,
                            // stencil_read_only: false,
                        }
                    }),
                });
                pass
            }
        }
    }
}
