hl = pixelCount / 2
export function beforeRender(delta) {
  t1 = time(.1)
  t2 = time(0.13)
}

export function render(index) {
  c1 = 1 - abs(index - hl) / hl
  c2 = wave(c1)
  c3 = wave(c2 + t1)
  v = wave(c3 + t1)
  v = v * v
  hsv(c1 + t2, 1, v)
}
