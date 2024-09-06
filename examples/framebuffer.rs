use winit::{
    self,
    event::{self, Event, WindowEvent},
};

fn frame(device: &wgpu::Device, framebuffer: &mut wgpu_misc::Framebuffer) -> wgpu::CommandBuffer {
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    {
        let _pass = framebuffer.begin_render_pass(&mut encoder);
    }

    encoder.finish()
}

async fn app() {
    let instance = wgpu::Instance::new(Default::default());
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: false, //Some(&surface),
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
            },
            None,
        )
        .await
        .unwrap();
    let builder = winit::window::WindowBuilder::new();
    let event_loop = winit::event_loop::EventLoop::new().expect("Event loop");
    let window = std::sync::Arc::new(builder.with_visible(false).build(&event_loop).unwrap());

    let mut framebuffer = wgpu_misc::Framebuffer::new_from_window(
        &instance,
        window.clone(),
        wgpu::TextureFormat::Bgra8UnormSrgb,
    );

    let mut tex_fb = wgpu_misc::Framebuffer::new_with_texture(wgpu::TextureFormat::Rgba8UnormSrgb);

    tex_fb.set_clear_color(&[0.2, 0.3, 0.7, 1.0]);

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

    let _s = event_loop.run(move |event, elwt| match event {
        Event::AboutToWait => {
            window.request_redraw();
        }
        Event::WindowEvent {
            event: WindowEvent::Resized(size),
            ..
        } => {
            framebuffer.set_resolution(size.width, size.height);
            framebuffer.configure(&device);
        }
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::RedrawRequested => {
                let cmd_buf = frame(&device, &mut framebuffer);

                queue.submit(Some(cmd_buf));
                framebuffer.present();
            }
            WindowEvent::KeyboardInput {
                event:
                    event::KeyEvent {
                        logical_key: winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape),
                        state: event::ElementState::Pressed,
                        ..
                    },
                ..
            }
            | event::WindowEvent::CloseRequested => {
                elwt.exit();
            }
            _ => {}
        },

        _ => {}
    });
}

fn main() {
    wgpu_misc::block_on(app());
}
