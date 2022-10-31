# 3 languages in a trenchcoat

A questionable combination of JavaScript (in syntax), FORTH (in spirit) and MicroPython (in terms of scope and being a wild mix of weird and kinda cool).

## Dear `$deity`, why?

- Hot code reloading on embedded without having to flash a whole new binary: especially on `esp32-idf` image size and thus turnaround time can be a bit of an obstacle.
- Port [Pixelblaze](https://www.bhencke.com/pixelblaze) to Rust.

# See it in action

this is the web app, JavaScript editor + live updated in-browser rendering of the code, alongside hot code reload sent to a `no_std`,`no_alloc` microcontroller
![](media/showcase.gif)

## Features 

- *extremely* fast turnaround, no noticable delay between code change and web app/mcu update

- Care has been taken to keep runtime platform, language and language dialect generic. This means:
  - runtime: you can run `trenchcoat` on a PC, a microcontroller, or in the browser.
  - language dialect: Pixelblaze-specific JavaScript extensions are factored out and don't pollute the standard JS namespace
  - language: the virtual machine actually executing code is a language-agnostic stack machine, there just happened to be a [JavaScript parser](https://rustdoc.swc.rs/swc_ecma_parser/) lying around. If you want to add, say, Python syntax support, you totally can! I won't! (Pull requests are welcome, though)

## Limitations
- Only a very minimal subset of JavaScript and Pixelblaze functionality is supported. You want `for` loops? Maybe in the next release…
- Extremely unoptimized! Also, basically no prior art has been considered so it's probably full of Arrogant Rookie™ mistakes.
- Parsing is not available on microcontrollers (so, no on-device REPL). The architecture allows implementing it, though.
- the existing `no_std` embedded apps don't use an allocator, which means a lot of wasted memory for static buffer headroom. Allocator support is implemented but not currently used.
- The license needs to be piped through a lawyer.

## Enough talking, how do I run this?

Right now the main focus lies on getting pixelblaze support to mature, so that's also what these instructions will focus on.

The general approach is:

1. Pick a runtime (console/web/embedded) and compile JavaScript/Pixelblaze source to bytecode
2. For embedded only: pick an update path - the web app uses inline compilation + HTTP to UART updates for hot code reload, but if you don't need that, you can also use the bundled `console-compiler` to compile bytecode to disk (`.tcb` for "TrenChcoat Bytecode" is a suggested file extension) and "somehow" have your firmware access it, e.g. via `include_bytes!`. If you want to update via http but your mcu is connected via UART (e.g. the bundled `stm32f4-app`), launch `http-to-serial.py /dev/YOUR-SERIAL-DEVICE` as a bridge.
3. Spawn an `Executor`, `start()` it once and call `do_frame()` as many times as you wish to produce LED colors. On `no_std` "current time" needs to be advanced manually from some timer source (the example app reuses the frame task's scheduling interval). `Executor::exit()` is optional.

Feature flags to pick:
- Desktop: `["full"]`
- Web app/wasm: `["compiler", "log", "use-std"]`
- Embedded: `["defmt"]` or `["defmt", "alloc"]` when you have an allocator
  - esp32 with IDF (`std` support): `["log", "use-std"]`

### WeAct STM32F4x1 aka "USB-C pill", "black pill" 

- you need a working hardware probe + `probe-run` setup.
- the example app uses SPI2 on PB15 with 16 WS2812 LEDs. Most heavy LED lifting is done in the adjacent `f4-peri` crate; you can also use SPI1+PB5 by using the `spi` feature instead of the default `spi_alt`. `f4-peri` also supports the SK9822/APA102 protocol if you prefer a more stable LED.

```shell
cd console-compiler
# rainbow melt is the only verified-working file at the moment
cargo run -- -f pixelblaze -i ../res/rainbow\ melt.js -o "../res/rainbow melt.tcb" 
cd ../stm32f4-app
# probe-run is required
cargo rrb app
```

### Espressif C3 (and potentially S2)
TODO, up next!

### Raspberry Pi Pico
TODO, up next!
### Browser

(*a cool bear spawns from an adjacent universe*)

**cool bear:** Browser? As in ... you're running a rudimentary JavaScript virtual machine ... in the browser ...

**author, in straightjacket:** you got that exactly right. With no performance-boosting offload support whatsoever!

On the bright side, we don't need a separate compilation step as part of our build. 
Because *the compiler also runs in the browser, muahahaha*

See `main.rs` for more details. Currently the web app is wildly inefficient (but still plenty fast), and also the editor permanently loses cursor position. WIP.

```shell
cargo install --git https://github.com/DioxusLabs/cli # their stable version seems broken atm
cd web-app
dioxus serve
$browser http://localhost:8080/
```

## Acknowledgements
- Forth-ish VM inspired by [forth-rs](https://github.com/dewaka/forth-rs) 
- Abstract (`log`/`defmt`) logging macros courtesy of [dirbaio](https://github.com/Dirbaio) and [whitequark](https://github.com/whitequark)
- Cool Bear™ by [fasterthanlime](https://fasterthanli.me/)

## Resources
[Pixelblaze expression language](https://github.com/simap/pixelblaze/blob/master/README.expressions.md)
