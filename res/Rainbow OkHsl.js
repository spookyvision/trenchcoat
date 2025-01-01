var colorMod = 0.5
var speed = 0.5
var direction = 1
var saturation = 1
var lightness = 0.64

export function sliderColorMod(v) { colorMod = v - 1 }
export function sliderSpeed(v) { speed = v }
export function sliderDirection(v) {
  if (v < 0.5) {
    direction = -1
  } else {
    direction = 1
  }
}
export function sliderSaturation(v) { saturation = v }
export function sliderLightness(v) { lightness = v }

export function beforeRender(_delta) {
  actual_speed = 1 - speed * 0.999;
  t1 = time(0.1 * actual_speed * direction)
}

export function render(index) {
  h = t1 + index / pixelCount * colorMod * 2
  s = saturation
  l = lightness
  ext_okhsl(h, s, l)
}
