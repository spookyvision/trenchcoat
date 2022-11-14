use std::{
    sync::{Arc, Mutex},
    thread::{sleep, JoinHandle},
    time::{Duration, Instant},
};

use trenchcoat::{
    forth::vm::VM,
    pixelblaze::{executor::Executor, ffi::PixelBlazeFFI},
};
mod runtime;
use crate::{app_config::AppConfig, runtime::EspRuntime};

pub(crate) mod bsc;
#[cfg(feature = "ws2812")]
pub(crate) mod ws_peri;
use embedded_svc::httpd::{registry::Registry, Method, Response};
use esp_idf_svc::httpd;
// If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
// ivmarkov: "binstart gives you a regular Rust STD fn main()"
use log::{info, warn};

pub(crate) mod app_config;

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    info!("trenchcoat!");

    let sys_start = Instant::now();
    let config = AppConfig::new();
    let _wifi = bsc::wifi::wifi(&config.wifi_ssid, &config.wifi_psk)?;

    let (_httpd, _vm_thread) = httpd(&config)?;

    loop {
        sleep(Duration::from_millis(10));
    }
}

fn httpd(config: &AppConfig) -> anyhow::Result<(httpd::Server, JoinHandle<()>)> {
    let mut vm = VM::new_empty(EspRuntime::default());
    vm.runtime_mut().init(config);
    let mut executor = Executor::new(vm, config.pixel_count);
    executor.start();
    let executor = Arc::new(Mutex::new(executor));

    let frame_ex = executor.clone();
    let vm_thread_handle = std::thread::spawn(move || loop {
        if let Ok(mut executor) = frame_ex.lock() {
            executor.do_frame();
        }
        sleep(Duration::from_millis(50));
    });

    let server = httpd::ServerRegistry::new()
        .at("/")
        .post(move |mut request| {
            info!("got new vm!");
            if let Ok(mut ser_vm) = request.as_bytes() {
                if let Ok(mut ex_handle) = executor.lock() {
                    let mut next_vm =
                        postcard::from_bytes_cobs::<VM<PixelBlazeFFI, EspRuntime>>(&mut ser_vm)?;
                    let runtime = ex_handle.take_vm().unwrap().dismember();
                    *next_vm.runtime_mut() = runtime;
                    ex_handle.set_vm(next_vm);
                    info!("restarting vm!");
                    ex_handle.start();
                }
            }
            let response = Response::ok();
            response
                .header("Access-Control-Allow-Origin", "*")
                .header("Content-type", "text/plain")
                .into()
        })?
        .at("/")
        .handler(Method::Options, |_request| {
            let response = Response::ok();
            response
                .header("Access-Control-Allow-Origin", "*")
                .header("Content-type", "text/plain")
                .into()
        })?;

    let server = server.start(&Default::default());

    server.map(|server| (server, vm_thread_handle))
}
