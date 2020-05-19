use wgpu;

use wgpu_util;
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
    let instance = wgpu::Instance::new();
    let adapter = instance
        .request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: None, //Some(&surface),
            },
            wgpu::BackendBit::PRIMARY,
        )
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                extensions: wgpu::Extensions {
                    anisotropic_filtering: false,
                },
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

    let mut framebuffer = wgpu_util::Framebuffer::new(&instance, &window);

    let sz = window.inner_size();
    framebuffer.resize(&device, sz.width, sz.height, true);

    framebuffer.set_clear_color(&[0.7, 0.3, 0.2, 1.0]);

    // Render first frame, and make window visible only after,
    // so we don't get a flash of empty window
    // TODO: It seems it is enough to just have a set_visible call, as
    // long as it was created as hidden, but need to check other platforms
    // (tested on osx)
    let cmd_buf = frame(&device, &mut framebuffer);
    queue.submit(Some(cmd_buf));
    window.set_visible(true);

    event_loop.run(move |event, _, control_flow| match event {
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        Event::RedrawRequested(_) => {
            let cmd_buf = frame(&device, &mut framebuffer);

            queue.submit(Some(cmd_buf));
        }
        Event::WindowEvent {
            event: WindowEvent::Resized(size),
            ..
        } => {
            framebuffer.resize(&device, size.width, size.height, true);
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
