pub struct Framebuffer {
    surface: wgpu::Surface,
    swap_chain: Option<wgpu::SwapChain>,
    frame: Option<wgpu::SwapChainOutput>,
    depth_texture_view: Option<wgpu::TextureView>,
    clear_color: [f64; 4],
    resolution: (u32, u32),
}

impl Framebuffer {
    pub fn new<W: raw_window_handle::HasRawWindowHandle>(
        instance: &wgpu::Instance,
        window: &W,
    ) -> Framebuffer {
        let surface = unsafe { instance.create_surface(window) };

        Framebuffer {
            surface,
            swap_chain: None,
            frame: None,
            depth_texture_view: None,
            clear_color: [0f64, 0f64, 0f64, 1f64],
            resolution: (0, 0),
        }
    }

    pub fn set_clear_color(&mut self, clear_color: &[f64; 4]) {
        self.clear_color = *clear_color;
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32, depth_enabled: bool) {
        if width == self.resolution.0
            && height == self.resolution.1
            && depth_enabled == self.depth_texture_view.is_some()
        {
            // Nothing changed
            return;
        }

        self.frame = None;

        self.resolution = (width, height);
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            // TODO: Allow srgb unconditionally
            format: if cfg!(target_arch = "wasm32") {
                wgpu::TextureFormat::Bgra8Unorm
            } else {
                wgpu::TextureFormat::Bgra8UnormSrgb
            },
            width: width,
            height: height,
            present_mode: wgpu::PresentMode::Mailbox,
        };
        let swap_chain = device.create_swap_chain(&self.surface, &sc_desc);
        self.swap_chain = Some(swap_chain);

        if depth_enabled {
            self.depth_texture_view = Some(
                device
                    .create_texture(&wgpu::TextureDescriptor {
                        size: wgpu::Extent3d {
                            width: width,
                            height: height,
                            depth: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Depth24Plus,
                        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                        label: Some("wgpu-util depth texture"),
                    })
                    .create_default_view(),
            );
        }
    }

    pub fn begin_render_pass<'a>(
        &'a mut self,
        encoder: &'a mut wgpu::CommandEncoder,
    ) -> wgpu::RenderPass<'a> {
        // The lifetimes above are telling that the RenderPass must not be
        // dropped before self or encoder, as the RenderPass will refer to
        // values in them

        let swap_chain = self.swap_chain.as_mut().expect(
            "swap chain is missing, did you remember to call `resize` before `begin_render_pass`",
        );
        self.frame = None;
        let frame = swap_chain
            .get_next_texture()
            .expect("Timeout when acquiring next swap chain texture");

        self.frame = Some(frame);
        let frame_view = &self.frame.as_mut().unwrap().view;

        let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: frame_view,
                resolve_target: None,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: wgpu::Color {
                    r: self.clear_color[0],
                    g: self.clear_color[1],
                    b: self.clear_color[2],
                    a: self.clear_color[3],
                },
            }],
            depth_stencil_attachment: self.depth_texture_view.as_ref().map(|tex| {
                wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: tex,
                    depth_load_op: wgpu::LoadOp::Clear,
                    depth_store_op: wgpu::StoreOp::Store,
                    stencil_load_op: wgpu::LoadOp::Clear,
                    stencil_store_op: wgpu::StoreOp::Store,
                    clear_depth: 1.0,
                    clear_stencil: 0,
                }
            }),
        });
        pass
    }
}
