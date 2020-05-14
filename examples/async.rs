
#[cfg(feature="wgpu-05")]
use wgpu_05_dep as wgpu;
#[cfg(feature="wgpu-master")]
use wgpu_master_dep as wgpu;

use wgpu_util;

async fn app() {

    #[cfg(feature="wgpu-master")]
    {
        let instance = wgpu::Instance::new();
        let _adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: None //Some(&surface),
            },
            wgpu::BackendBit::PRIMARY,
        )
        .await
        .unwrap();
    
        // unimplemented!() in master
        // println!("Adapter: {}", adapter.get_info().name);
    }
    #[cfg(feature="wgpu-05")]
    {
        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: None //Some(&surface),
            },
            wgpu::BackendBit::PRIMARY,
        )
        .await
        .unwrap();

        println!("Adapter: {}", adapter.get_info().name);
    }


}


fn main()
{

    wgpu_util::block_on(app);

}
