use std::{
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};

use embedded_svc::io::Write;
use esp_idf_hal::prelude::Peripherals;
use trenchcoat::{
    forth::vm::VM,
    pixelblaze::{executor::Executor, ffi::PixelBlazeFFI},
};
mod runtime;
use crate::{app_config::AppConfig, runtime::EspRuntime};

pub(crate) mod bsc;
#[cfg(feature = "ws2812")]
pub(crate) mod ws_peri;

use embedded_svc::{
    http::{server::Method, Headers},
    io::Read,
};
use esp_idf_svc::http::server::EspHttpServer;
use log::info;

pub(crate) mod app_config;

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    info!("trenchcoat!");

    let peripherals = Peripherals::take().unwrap();

    let config = AppConfig::new();
    let _wifi = bsc::wifi::wifi(&config.wifi_ssid, &config.wifi_psk, peripherals.modem)?;

    info!("starting VM");

    let mut vm = VM::new_empty(EspRuntime::default());
    vm.runtime_mut().init(&config);
    let mut executor = Executor::new(vm, config.pixel_count);
    executor.start();
    let executor = Arc::new(Mutex::new(executor));

    let frame_ex = executor.clone();

    info!("starting web server");
    let _httpd = httpd(executor.clone())?;

    loop {
        if let Ok(mut executor) = frame_ex.lock() {
            executor.do_frame();
        }
        sleep(Duration::from_millis(10));
    }
}

const CORS_HEADERS: [(&str, &str); 2] = [
    ("Access-Control-Allow-Origin", "*"),
    ("Content-type", "text/plain"),
];

fn httpd(
    executor: Arc<Mutex<Executor<PixelBlazeFFI, EspRuntime>>>,
) -> anyhow::Result<EspHttpServer> {
    let mut server = EspHttpServer::new(&Default::default())?;

    server
        .fn_handler("/", Method::Post, move |mut request| {
            if let Some(len) = request.header("Content-Length") {
                let len: usize = len.parse()?;
                info!("body: {len} bytes");
                let mut body: Vec<u8> = Vec::with_capacity(len);
                body.resize(len, 0);
                request.read(&mut body)?;
                if let Ok(mut ex_handle) = executor.lock() {
                    info!("loading bytecode");
                    let mut next_vm =
                        postcard::from_bytes_cobs::<VM<PixelBlazeFFI, EspRuntime>>(&mut body)?;
                    info!("updating VM");
                    let runtime = ex_handle.take_vm().unwrap().dismember();
                    *next_vm.runtime_mut() = runtime;
                    ex_handle.set_vm(next_vm);
                    info!("restarting VM");
                    ex_handle.start();
                }
            }
            request
                .into_response(200, Some("OK"), &CORS_HEADERS)?
                .write_all("ÖK".as_bytes())?;
            Ok(())
        })?
        .fn_handler("/", Method::Options, |request| {
            request
                .into_response(200, Some("OK"), &CORS_HEADERS)?
                .write_all(b"")?;
            Ok(())
        })?;

    Ok(server)
}
