// An XOR in 2D/3D space based on block reflections

export function beforeRender(delta) {
  t2 = time(0.1) * PI2
  t1 = time(.1)
  t3 = time(.5)
  t4 = time(0.2) * PI2
}

export function render2D(index, x, y) {
  render3D(index, x, y, 0)
}

export function render3D(index, x, y, z) {
  h = sin(t2)
  m = (.3 + triangle(t1) * .2)
  h = h + (wave((5*(x-.5) ^ 5*(y-.5) ^ 5*(z-.5))/50  * ( triangle(t3) * 10 + 4 * sin(t4)) % m))
  s = 1;
  v = ((abs(h) + abs(m) + t1) % 1);
  v = triangle(v*v)
  h = triangle(h)/5 + (x + y + z)/3 + t1
  v = v * v * v
  hsv(h, s, v)
}
