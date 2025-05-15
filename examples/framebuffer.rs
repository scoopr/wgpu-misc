use winit::{self, event::WindowEvent};

fn frame(device: &wgpu::Device, framebuffer: &mut wgpu_misc::Framebuffer) -> wgpu::CommandBuffer {
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    {
        let _pass = framebuffer.begin_render_pass(&mut encoder);
    }

    encoder.finish()
}

struct Example {
    window: std::sync::Arc<winit::window::Window>,
    framebuffer: wgpu_misc::Framebuffer,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl Example {
    fn new(event_loop: &winit::event_loop::ActiveEventLoop) -> Self {
        let window_attributes = winit::window::WindowAttributes::default().with_visible(false);
        let window = std::sync::Arc::new(event_loop.create_window(window_attributes).unwrap());
        let window2 = window.clone(); // moved to async block
        let (device, queue, surface) = wgpu_misc::block_on(async move {
            let instance = wgpu::Instance::new(&Default::default());
            let surface = instance.create_surface(window2).expect("surface");

            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::LowPower,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false, //Some(&surface),
                })
                .await
                .expect("Adapter request");

            let (device, queue) = adapter
                .request_device(&wgpu::DeviceDescriptor {
                    label: Some("device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::default(),
                    trace: wgpu::Trace::Off,
                })
                .await
                .expect("Device request");

            (device, queue, surface)
        });

        let mut framebuffer =
            wgpu_misc::Framebuffer::new_from_surface(surface, wgpu::TextureFormat::Bgra8UnormSrgb);
        let sz = window.inner_size();
        framebuffer.set_resolution(sz.width, sz.height);
        framebuffer.set_depth_stencil_format(Some(wgpu::TextureFormat::Depth24Plus));
        framebuffer.configure(&device);

        framebuffer.set_clear_color(&[0.7, 0.3, 0.2, 1.0]);

        // Render first frame, and make window visible only after,
        // so we don't get a flash of empty window
        // TODO: It seems it is enough to just have a set_visible call, as
        // long as it was created as hidden, but need to check other platforms
        // (tested on osx)
        let cmd_buf = frame(&device, &mut framebuffer);

        queue.submit(Some(cmd_buf));
        framebuffer.present();
        window.set_visible(true);

        Self {
            window,
            framebuffer,
            device,
            queue,
        }
    }
}

struct ExampleApp {
    example: Option<Example>,
}

impl winit::application::ApplicationHandler for ExampleApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.example.is_none() {
            self.example = Some(Example::new(event_loop));
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Some(ref mut example) = self.example else {
            return;
        };
        match event {
            WindowEvent::Resized(size) => {
                example.framebuffer.set_resolution(size.width, size.height);
                example.framebuffer.configure(&example.device);
            }
            WindowEvent::RedrawRequested => {
                let cmd_buf = frame(&example.device, &mut example.framebuffer);

                example.queue.submit(Some(cmd_buf));
                example.framebuffer.present();
            }
            WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        logical_key: winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape),
                        state: winit::event::ElementState::Pressed,
                        ..
                    },
                ..
            }
            | winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        let Some(ref mut example) = self.example else {
            return;
        };
        example.window.request_redraw();
    }
}

fn main() {
    let event_loop = winit::event_loop::EventLoop::new().expect("Event loop");

    let mut tex_fb = wgpu_misc::Framebuffer::new_with_texture(wgpu::TextureFormat::Rgba8UnormSrgb);

    tex_fb.set_clear_color(&[0.2, 0.3, 0.7, 1.0]);

    let mut app = ExampleApp { example: None };

    event_loop.run_app(&mut app).expect("Event loop run");
}
