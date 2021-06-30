async fn app() {
    let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None, //Some(&surface),
        })
        .await
        .unwrap();

    println!("Adapter: {}", adapter.get_info().name);
}

fn main() {
    wgpu_misc::block_on(app);
}
