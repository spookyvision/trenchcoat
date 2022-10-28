# 3 languages in a trenchcoat

A questionable mix of JavaScript (in syntax), FORTH (in spirit) and MicroPython (in terms of scope and being a wild mix of weird and kinda cool).

## Dear `$deity`, why?

- Hot code reloading on embedded without having to flash a whole new binary: especially on `esp32-idf` image size and thus turnaround time can be a bit of an obstacle.
- Port [Pixelblaze](https://www.bhencke.com/pixelblaze) to Rust.

## Features 

Care has been taken to keep runtime platform, language and language dialect generic. This means:
- runtime: you can run `trenchcoat` on a PC, a microcontroller, or in the browser.
- language dialect: Pixelblaze-specific JavaScript extensions are factored out and don't pollute the standard JS namespace
- language: the virtual machine actually executing code is a language-agnostic stack machine, there just happened to be a [JavaScript parser](https://rustdoc.swc.rs/swc_ecma_parser/) lying around. If you want to add, say, Python syntax support, you totally can! I won't!

## Limitations
- Only a very minimal subset of JavaScript and Pixelblaze functionality is supported. You want `for` loops? Maybe in the next release…
- Extremely unoptimized! Also, basically no prior art has been considered so it's probably full of Arrogant Rookie™ mistakes.
- Parsing is not available on microcontrollers (so, no on-device REPL). The architecture allows implementing it, though.
- The license needs to be piped through a lawyer.

## Enough talking, how do I run this?

TODO

## Acknowledgements
- Forth-ish VM inspired by [forth-rs](https://github.com/dewaka/forth-rs) 
- Abstract (`log`/`defmt`) logging macros courtesy of [dirbaio](https://github.com/Dirbaio) and [whitequark](https://github.com/whitequark)

## Resources
[Pixelblaze expression language](https://github.com/simap/pixelblaze/blob/master/README.expressions.md)