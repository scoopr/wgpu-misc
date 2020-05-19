use wgpu;

use wgpu_util;

async fn app() {
    let instance = wgpu::Instance::new();
    let _adapter = instance
        .request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: None, //Some(&surface),
            },
            wgpu::BackendBit::PRIMARY,
        )
        .await
        .unwrap();

    // unimplemented!() in master
    // println!("Adapter: {}", adapter.get_info().name);
}

fn main() {
    wgpu_util::block_on(app);
}
