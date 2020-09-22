use wgpu;

use wgpu_util;

async fn app() {
    let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::Default,
            compatible_surface: None, //Some(&surface),
        })
        .await
        .unwrap();

    println!("Adapter: {}", adapter.get_info().name);
}

fn main() {
    wgpu_util::block_on(app);
}
