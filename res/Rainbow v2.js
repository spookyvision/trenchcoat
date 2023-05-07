var colorMod = 1
var speed = 1
var direction = 1
var saturation = 1
var lightness = 1

export function sliderColorMod(v) { colorMod = v - 1 }
export function sliderSpeed(v) { speed = v }
export function sliderDirection(v) {
  // if(v < 0.5) {
  direction = -1
  // } else {
  direction = 1
  // }
}
export function sliderSaturation(v) { saturation = v }
export function sliderLightness(v) { lightness = v }

export function beforeRender(delta) {
  t1 = time(.1 * speed * direction)
}

export function render(index) {
  h = t1 + index / pixelCount * colorMod
  s = saturation
  v = lightness
  hsv(h, s, v)
}
