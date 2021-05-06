use winit::{
    self,
    event::{
        self,
        Event::{self},
        WindowEvent,
    },
    event_loop,
};

fn frame(device: &wgpu::Device, framebuffer: &mut wgpu_util::Framebuffer) -> wgpu::CommandBuffer {
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    {
        let _pass = framebuffer.begin_render_pass(&mut encoder);
    }

    encoder.finish()
}

async fn app() {
    let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None, //Some(&surface),
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("device"),
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            /*        match trace_dir {
                Ok(ref value) if !cfg!(feature = "trace") => {
                    log::error!("Unable to trace into {:?} without \"trace\" feature enabled!", value);
                    None
                }
                Ok(ref value) => Some(std::path::Path::new(value)),
                Err(_) => None,
            },*/
            None,
        )
        .await
        .unwrap();
    let builder = winit::window::WindowBuilder::new();
    let event_loop = winit::event_loop::EventLoop::new();
    let window = builder.with_visible(false).build(&event_loop).unwrap();

    let mut framebuffer = wgpu_util::Framebuffer::new_from_window(
        &instance,
        &window,
        wgpu::TextureFormat::Bgra8UnormSrgb,
    );

    let mut tex_fb = wgpu_util::Framebuffer::new_with_texture(wgpu::TextureFormat::Rgba8UnormSrgb);

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

    queue.submit(vec![cmd_buf]);
    framebuffer.present();
    window.set_visible(true);

    event_loop.run(move |event, _, control_flow| match event {
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        Event::RedrawRequested(_) => {
            let cmd_buf = frame(&device, &mut framebuffer);

            queue.submit(Some(cmd_buf));
            framebuffer.present();
        }
        Event::WindowEvent {
            event: WindowEvent::Resized(size),
            ..
        } => {
            framebuffer.set_resolution(size.width, size.height);
            framebuffer.configure(&device);
        }
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::KeyboardInput {
                input:
                    event::KeyboardInput {
                        virtual_keycode: Some(event::VirtualKeyCode::Escape),
                        state: event::ElementState::Pressed,
                        ..
                    },
                ..
            }
            | event::WindowEvent::CloseRequested => {
                *control_flow = event_loop::ControlFlow::Exit;
            }
            _ => {}
        },

        _ => {}
    });
}

fn main() {
    wgpu_util::block_on(app);
}
