async fn app() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        ..Default::default()
    });
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: false, //Some(&surface),
        })
        .await
        .unwrap();

    println!("Adapter: {}", adapter.get_info().name);
}

fn main() {
    wgpu_misc::block_on(app());
}
