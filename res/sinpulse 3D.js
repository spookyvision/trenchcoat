
export function beforeRender(delta) {
  t1 = time(.05)*PI2
  t2 = time(.09)*PI2
  zoom = 1+ wave(time(.2))*3
  t3 = time(.1)
}

export function render(index) {
  render3D(index, index/pixelCount, 0, 0)
}

export function render2D(index, x, y) {
  render3D(index, x, y, 0)
}

export function render3D(index, x, y, z) {

  h = (1 + sin(x*zoom + t1) + cos(y*zoom + t2) + sin(z*zoom + t1 - t2))*.5
  v = h
  v = v*v*v
  hsv(h,1,v/2)
}
