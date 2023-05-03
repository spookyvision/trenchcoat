var red = 0.1
var green = 0.7
var ison = false;

export function sliderRed(v) {
  red = v
}

export function sliderGreen(v) {
  green = v
}

export function toggleIson(v) {
  ison = v
}

export function beforeRender(delta) {
  t1 = time(.1)
}

export function render(index) {
  rgb(red, green, t1)
}
