/*
  Perlin noise is procedurially generated gradient noise. It's 2D output can be 
  thought of as a random topo map. This pattern is optimized for 2D layouts, not strips.
  
  Demo on a 16x16 matrix:
    https://youtu.be/-dahz1u9DXM
  Mapper map for four 8x8 matrices:
    https://gist.github.com/jvyduna/a899c5e6eacddb90257a3e8b55fd1fda
  8x8 WS2812b matix ($14 ea): 
    https://www.tindie.com/products/electromage/electromage-8x8-led-matrix/
  
  perlin2() is based on a square grid.
  simplex2() is based on a triangular grid.
  
  This is a large, complex pattern that's halted on me while developing. If it hangs, 
  play with user controls (e.g. is Stripe Speed 0?) or reload PB and reselect the pattern.
  
  The Perlin noise code was adapted from
    https://github.com/josephg/noisejs/blob/master/perlin.js
  
  That code was placed in the public domain by its original author, Stefan Gustavson. 
  You may use it as you see fit, but attribution is appreciated.
*/

// Setup constants
var width = 16 // Width (in pixels) of your matrix
var height = pixelCount / width
var pixels = array(width) // This will store the height vales of the Perlin noise field
for (i = 0; i < width; i++) pixels[i] = array(height) // Allocate 2D array

// User variables
var useSimplex = 1, scale = 2, motion = 0, panning = 0, autoColor = 1, colorOffset = 0.4 
var stripes = 3, sweepSpeed = 1, subStripes = 1, x0 = 0, y0 = 0, showProgressBar = 0
var bassThreshold = 0
export function sliderPerlinOrSimplex(v) { useSimplex = v > 0.5 } // Larger = finer grained noise
export function sliderScale(v) { scale = 1 + v * 4; needsRecalc = 1 } // Larger = finer grained noise
export function sliderMotion(v) { panning = v } // Circular viewport panning speed
export function sliderAutoColor(v) { autoColor = v > 0.5 } // Colormode 0 uses specific user-defined colors
export function sliderAutoColorPalette(h) { colorOffset = h }
export function sliderNumberOfStripes(v) { stripes = 1 + floor(v * 4) } // Number of Stripes that sweep through contours
export function sliderStripeSpeed(v) { sweepSpeed = v } // Speed of stripe flow
export function sliderStripeWeight(v) { subStripes = 5 - floor(v * 4) } // Number of slots in a stripe - we'll blank out all but 1
export function sliderX_Offset(x) { x0 = x * scale * 4; needsRecalc = 1 } // x translation (panning)
export function sliderY_Offset(y) { y0 = y * scale * 4; needsRecalc = 1 } // y translation (panning)
export function sliderShowProgress(v) { showProgressBar = v > 0.5 } // Display a white progress bar
export function sliderBassThreshold(v) { bassThreshold = v * 0.03 } // Enable bass reactive flow. Higher = bass must be louder.
seed = 30  // Perlin seed - which random heightmap we sweep through
var bassDuration = 160  // Duration of the fast-forward upon bass triger (in ms)
// Only used if autoColor = 0. Colors must be in ascending order. Here's "fire".
color1 = 0; color2 = 0.028; color3 = 0.07

// Global state
needsRecalc = 1 // When there's translation or scaling, we need to recalc the whole field
var bassPlaying, bassTimeAccum, t1SoundOffset // Stuff for sound accelleration
export var frequencyData  // Enable sensor expansion board sound spectrum

// Accelerate timer t1 when we've detected bass for bassDuration ms
function reactToSound(delta) {
  bassTrigger = (frequencyData[2] + frequencyData[3] + 
                 frequencyData[4] + frequencyData[5]) > bassThreshold
  bassPlaying = bassPlaying || bassTrigger
  if (bassPlaying) {
    bassTimeAccum += delta
    t1SoundOffset += (delta >> 11) * (1 + sweepSpeed)
    if (t1SoundOffset > 1) t1SoundOffset -= 1
    if (bassTimeAccum > bassDuration ) {
      bassTimeAccum = 0
      bassPlaying = 0
    }
  }
}

export function beforeRender(delta) {
  t1 = time(0.44 / sweepSpeed * stripes / 65.536)   // Speed of sweeping countours
  if (bassThreshold) reactToSound(delta)
  t1 = (t1 + t1SoundOffset) % 1

  t2 = time(100 / 65.536)                        // Speed of viewport camera panning
  tx = panning * 40 / scale * sin(PI2 * t2) / 2  // x translation (panning)
  ty = panning * 40 / scale * cos(PI2 * t2) / 2  // y translation (panning)
  
  if (needsRecalc || panning) {
    recalculate(tx + x0, ty + y0)
    needsRecalc = 0
  }
}

var minHeight, maxHeight, altRange
function recalculate(offsetX, offsetY) {
  minHeight = maxHeight = 0
  for (x = 0; x < width; x++) {
    for (y = 0; y < height; y++) {
      noiseFunc = useSimplex ? simplex2 : perlin2
      pixels[x][y] = noiseFunc(
          (x / width - offsetX) * scale, 
          (y / height - offsetY) * scale
        )
      minHeight = min(minHeight, pixels[x][y])
      maxHeight = max(maxHeight, pixels[x][y])
    }
  }
  elevationRange = maxHeight - minHeight
}

export function render2D(index, x, y) {
  z = pixels[x * width][y * height] // These perlin noise map values can be in 0..1
  // n is the normalized height, guaranteed to go from 0..1 for every rendered map
  n = (z - minHeight) / elevationRange 
  
  if (autoColor) {
    // Assign a solid color to each sweeping stripe
    h = (floor(stripes * (n - t1)) / stripes ) % 1 - colorOffset
  } else {
    // Use three specified colors
    stripes = 3
    h = color1 + (color2 - color1) * square(n - t1 - 1/3, 2/3) + 
                 (color3 - color2) * square(n - t1 - 2/3, 1/3)
  }
  // Colored stripes sweeping along equal height contour lines
  v = triangle((n - t1) * subStripes * stripes)
  // Eliminate (subStripe - 1) of the subStripes to make a stripe more separated 
  // from neighbors and thus thinner
  v *= (floor((1 + n - t1) * subStripes * stripes) % subStripes == 0)
  
  hsv(h, 1, v * v)

  // Simple heightmap in blue for testing. Comment out for stripes.
  // hsv (0.66, 1, n * n) 
  
  if (showProgressBar) {
    // Superimpose a white progress bar showing t1 on the last row
    if (y >= (height - 1) / height && abs(t1 - x) * width < 2) {
      hsv(0, 0, 1 - clamp(abs((t1 - x) * width), 0, 1))
    }
  }
}



// Begin code from https://github.com/josephg/noisejs/blob/master/perlin.js
// 2D Perlin and Simplex Noise - Setup
var grad3 = array(12)
for (i = 0; i <12; i++) { grad3[i] = array(3) }

var gradIndex = 0
function grad(x, y, z) {
  grad3[gradIndex][0] = x
  grad3[gradIndex][0] = y
  grad3[gradIndex][0] = z
  gradIndex++
}

grad(1,1,0); grad(-1,1,0); grad(1,-1,0); grad(-1,-1,0)
grad(1,0,1); grad(-1,0,1); grad(1,0,-1); grad(-1,0,-1)
grad(0,1,1); grad(0,-1,1); grad(0,1,-1); grad(0,-1,-1)

var p = array(256)
p[0] = 151; p[1] = 160; p[2] = 137; p[3] = 91; p[4] = 90; p[5] = 15; p[6] = 131; p[7] = 13; p[8] = 201; p[9] = 95; p[10] = 96; p[11] = 53; p[12] = 194; p[13] = 233; p[14] = 7; p[15] = 225; p[16] = 140; p[17] = 36; p[18] = 103; p[19] = 30; p[20] = 69; p[21] = 142; p[22] = 8; p[23] = 99; p[24] = 37; p[25] = 240; p[26] = 21; p[27] = 10; p[28] = 23; p[29] = 190; p[30] =  6; p[31] = 148
p[32] = 247; p[33] = 120; p[34] = 234; p[35] = 75; p[36] = 0; p[37] = 26; p[38] = 197; p[39] = 62; p[40] = 94; p[41] = 252; p[42] = 219; p[43] = 203; p[44] = 117; p[45] = 35; p[46] = 11; p[47] = 32; p[48] = 57; p[49] = 177; p[50] = 33; p[51] = 88; p[52] = 237; p[53] = 149; p[54] = 56; p[55] = 87; p[56] = 174; p[57] = 20; p[58] = 125; p[59] = 136; p[60] = 171; p[61] = 168; p[62] =  68; p[63] = 175
p[64] = 74; p[65] = 165; p[66] = 71; p[67] = 134; p[68] = 139; p[69] = 48; p[70] = 27; p[71] = 166; p[72] = 77; p[73] = 146; p[74] = 158; p[75] = 231; p[76] = 83; p[77] = 111; p[78] = 229; p[79] = 122; p[80] = 60; p[81] = 211; p[82] = 133; p[83] = 230; p[84] = 220; p[85] = 105; p[86] = 92; p[87] = 41; p[88] = 55; p[89] = 46; p[90] = 245; p[91] = 40; p[92] = 244; p[93] =  102; p[94] = 143; p[95] = 54
p[96] =  65; p[97] = 25; p[98] = 63; p[99] = 161; p[100] =  1; p[101] = 216; p[102] = 80; p[103] = 73; p[104] = 209; p[105] = 76; p[106] = 132; p[107] = 187; p[108] = 208; p[109] =  89; p[110] = 18; p[111] = 169; p[112] = 200; p[113] = 196; p[114] = 135; p[115] = 130; p[116] = 116; p[117] = 188; p[118] = 159; p[119] = 86; p[120] = 164; p[121] = 100; p[122] = 109; p[123] = 198; p[124] = 173; p[125] = 186; p[126] =  3; p[127] = 64
p[128] = 52; p[129] = 217; p[130] = 226; p[131] = 250; p[132] = 124; p[133] = 123; p[134] = 5; p[135] = 202; p[136] = 38; p[137] = 147; p[138] = 118; p[139] = 126; p[140] = 255; p[141] = 82; p[142] = 85; p[143] = 212; p[144] = 207; p[145] = 206; p[146] = 59; p[147] = 227; p[148] = 47; p[149] = 16; p[150] = 58; p[151] = 17; p[152] = 182; p[153] = 189; p[154] = 28; p[155] = 42; p[156] = 223; p[157] = 183; p[158] = 170; p[159] = 213
p[160] = 119; p[161] = 248; p[162] = 152; p[163] =  2; p[164] = 44; p[165] = 154; p[166] = 163; p[167] =  70; p[168] = 221; p[169] = 153; p[170] = 101; p[171] = 155; p[172] = 167; p[173] =  43; p[174] = 172; p[175] = 9; p[176] = 129; p[177] = 22; p[178] = 39; p[179] = 253; p[180] =  19; p[181] = 98; p[182] = 108; p[183] = 110; p[184] = 79; p[185] = 113; p[186] = 224; p[187] = 232; p[188] = 178; p[189] = 185; p[190] =  112; p[191] = 104
p[192] = 218; p[193] = 246; p[194] = 97; p[195] = 228; p[196] = 251; p[197] = 34; p[198] = 242; p[199] = 193; p[200] = 238; p[201] = 210; p[202] = 144; p[203] = 12; p[204] = 191; p[205] = 179; p[206] = 162; p[207] = 241; p[208] =  81; p[209] = 51; p[210] = 145; p[211] = 235; p[212] = 249; p[213] = 14; p[214] = 239; p[215] = 107; p[216] =  49; p[217] = 192; p[218] = 214; p[219] =  31; p[220] = 181; p[221] = 199; p[222] = 106; p[223] = 157
p[224] = 184; p[225] =  84; p[226] = 204; p[227] = 176; p[228] = 115; p[229] = 121; p[230] = 50; p[231] = 45; p[232] = 127; p[233] =   4; p[234] = 150; p[235] = 254; p[236] = 138; p[237] = 236; p[238] = 205; p[239] = 93; p[240] = 222; p[241] = 114; p[242] = 67; p[243] = 29; p[244] = 24; p[245] = 72; p[246] = 243; p[247] = 141; p[248] = 128; p[249] = 195; p[250] = 78; p[251] = 66; p[252] = 215; p[253] = 61; p[254] = 156; p[255] = 180

var perm = array(512)
var gradP = array(512)


function perlinSeed(seed) {
  if (seed > 0 && seed < 1) {
    // Scale the seed out
    seed *= 32768
  }

  seed = floor(seed)
  if(seed < 256) {
    seed |= seed << 8
  }
  
  for(i = 0; i < 256; i++) {
    var perlinv
    if (i & 1) {
      perlinv = p[i] ^ (seed & 255)
    } else {
      perlinv = p[i] ^ ((seed>>8) & 255)
    }

    perm[i] = perm[i + 256] = perlinv
    gradP[i] = gradP[i + 256] = grad3[perlinv % 12]
  }
}
perlinSeed(seed)

function dot2(gradElement, x, y) {
  return gradElement[0] * x + gradElement[1] * y
}

// Skewing and unskewing factors for 2, 3, and 4 dimensions
var F2 = (sqrt(3) - 1) / 2
var G2 = (3 - sqrt(3)) / 6

// 2D simplex noise
// Note: Unlike a typical noise function -1..1, for Pixelblaze this outputs 0 to 1
function simplex2(xin, yin) {
  var n0, n1, n2 // Noise contributions from the three corners
  // Skew the input space to determine which simplex cell we're in
  var s = (xin + yin) * F2 // Hairy factor for 2D
  var i = floor(xin + s)
  var j = floor(yin + s)
  var t = (i + j) * G2
  var x0 = xin - i + t  // The x,y distances from the cell origin, unskewed.
  var y0 = yin - j + t
  // For the 2D case, the simplex shape is an equilateral triangle.
  // Determine which simplex we are in.
  var i1, j1  // Offsets for second (middle) corner of simplex in (i,j) coords
  if(x0>y0) { // lower triangle, XY order: (0,0)->(1,0)->(1,1)
    i1 = 1; j1 = 0
  } else {    // upper triangle, YX order: (0,0)->(0,1)->(1,1)
    i1 = 0; j1 = 1
  }
  // A step of (1,0) in (i,j) means a step of (1-c,-c) in (x,y), and
  // a step of (0,1) in (i,j) means a step of (-c,1-c) in (x,y), where
  // c = (3-sqrt(3))/6
  var x1 = x0 - i1 + G2  // Offsets for middle corner in (x,y) unskewed coords
  var y1 = y0 - j1 + G2
  var x2 = x0 - 1 + 2 * G2  // Offsets for last corner in (x,y) unskewed coords
  var y2 = y0 - 1 + 2 * G2
  // Work out the hashed gradient indices of the three simplex corners
  i &= 255
  j &= 255
  var gi0 = gradP[i+perm[j]];
  var gi1 = gradP[i+i1+perm[j+j1]];
  var gi2 = gradP[i+1+perm[j+1]];
  // Calculate the contribution from the three corners
  var pst0 = 0.5 - x0 * x0 - y0 * y0
  if (pst0 < 0) {
    n0 = 0
  } else {
    pst0 *= pst0
    n0 = pst0 * pst0 * dot2(gi0, x0, y0)  // (x,y) of grad3 used for 2D gradient
  }
  var pst1 = 0.5 - x1 * x1 - y1 * y1
  if (pst1 < 0) {
    n1 = 0
  } else {
    pst1 *= pst1
    n1 = pst1 * pst1 * dot2(gi1, x1, y1)
  }
  var pst2 = 0.5 - x2 * x2 - y2 * y2
  if (pst2 < 0) {
    n2 = 0
  } else {
    pst2 *= pst2
    n2 = pst2 * pst2 * dot2(gi2, x2, y2)
  }
  // Add contributions from each corner to get the final noise value.
  // The result is scaled to return values in the interval [0,1].
  return (70 * (n0 + n1 + n2) + 1) / 2
}

// Perlin noise functions
function fade(t) {
  return t * t * t * (t * (t * 6 - 15) + 10)
}

function lerp(a, b, t) {
  return (1 - t) * a + t * b
}

// 2D Perlin noise
// Note: Unlike a typical Perlin function, for Pixelblaze this outputs 0 to 1
function perlin2(x, y) {
  // Find unit grid cell containing point
  var X = floor(x)
  var Y = floor(y)
  // Get relative xy coordinates of point within that cell
  x = x - X; y = y - Y
  // Wrap the integer cells at 255 (smaller integer period can be introduced here)
  X = X & 255; Y = Y & 255

  // Calculate noise contributions from each of the four corners
  var n00 = dot2(gradP[X + perm[Y]], x, y)
  var n01 = dot2(gradP[X + perm[Y + 1]], x, y - 1)
  var n10 = dot2(gradP[X + 1 + perm[Y]], x - 1, y)
  var n11 = dot2(gradP[X + 1 + perm[Y + 1]], x - 1, y - 1)

  // Compute the fade curve value for x
  var u = fade(x)

  // Interpolate the four results
  return 0.5 + lerp(
    lerp(n00, n10, u),
    lerp(n01, n11, u),
    fade(y)
  )
}
