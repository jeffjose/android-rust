use dioxus::prelude::*;
use diretto::{Connector, Device};
use rustix::{
    fs::{self, Mode, OFlags},
    io,
};
use std::borrow::Cow;

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn start_app() {
    android_logger::init_once(
        android_logger::Config::default()
            .with_min_level(log::Level::Trace)
            .with_tag("template"),
    );

    dioxus_desktop::wry::android_binding!(com_example, template, _start_app, dioxus_desktop::wry);
}

#[cfg(target_os = "android")]
fn _start_app() {
    if let Err(err) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(main)) {
        eprintln!("attempt to unwind out of `rust` with err: {:?}", err);
        std::process::abort();
    }
}

#[cfg(not(target_family = "wasm"))]
pub fn main() {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    std::env::set_var("RUST_BACKTRACE", "1");
    dioxus_desktop::launch(app);
}

#[cfg(target_family = "wasm")]
pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();
    dioxus_web::launch(app);
}

fn app(cx: Scope) -> Element {
    cx.render(rsx! {
        h1 {
            "DRM SENSE"
        }

        h2 {
            format!("{}", get_info())
        }
    })
}

fn get_info() -> Result<&str, io::Errno> {
    let fd = fs::open(
        "/dev/dri/card0",
        OFlags::RDWR | OFlags::NONBLOCK,
        Mode::empty(),
    )?;
    let drm_device = unsafe { Device::new_unchecked(fd) };

    println!("Opened device /dev/dri/card0");

    let version = drm_device.version()?;

    println!(
        "Driver: {} ({}) version {}.{}.{} ({})",
        version.name.to_string_lossy(),
        version.desc.to_string_lossy(),
        version.major,
        version.minor,
        version.patchlevel,
        version.date.to_string_lossy()
    );

    let res = drm_device.get_resources()?;

    // Collect available connectors so we don't iterate again later
    let connectors = res
        .connectors
        .iter()
        .map(|id| drm_device.get_connector(*id, true))
        .collect::<io::Result<Vec<Connector>>>()?;

    for connector in &connectors {
        println!(
            "Found connector connecter_id: {} connector_type:{} connector_type_id:{}",
            connector.connector_id, connector.connector_type, connector.connector_type_id
        );

        for (i, (prop, prop_value)) in connector
            .props
            .iter()
            .zip(connector.prop_values.iter())
            .enumerate()
        {
            println!("  Prop: ({}:{})", prop, prop_value)
        }

        for mode in &connector.modes {
            println!(
                "  Found mode {}@{} for connector {}",
                mode.name().to_string_lossy(),
                mode.vertical_refresh_rate(),
                connector.connector_id
            )
        }
    }

    // Find the first connected monitor
    // FIXME: support more monitors
    let connector = connectors
        .into_iter()
        .find(|connector| connector.connection == 1) // 1 means connected
        .unwrap();

    // FIXME: The first mode is usually the prefered one but we should employ a better strategy
    let mode = connector.modes.first().expect("Connector has no modes");

    // This should somehow be passed to wgpu to choose the correct mode
    println!("Refresh rate: {}", mode.wsi_refresh_rate());

    let planes = drm_device.get_plane_resources()?;
    for plane in &planes {
        println!("Plane : {}", plane);
    }
    //Ok(version.name.to_string_lossy())
    Ok("foo")
}
