// Pew-Pew-Pew! v2.0.0 (Pattern for PixelBlaze)
// by Scott Balay -- https://github.com/zenblender

isForwardDirection = true // flip to run backwards
laserCount = 10  // use a multiple of numPaletteRGBs to have each available color represented equally
fadeFactor = 0.8
speedFactor = 0.01

// when on, new lasers cause entire strip to flash blue
// when off, blue component of each laser affects its color as normal
useBlueLightning = true

// init RGBs that in the palette of available colors:
numPaletteRGBs = 5
paletteRGBs = array(numPaletteRGBs)
paletteRGBs[0] = packRGB(255,13,107)
paletteRGBs[1] = packRGB(232,12,208)
paletteRGBs[2] = packRGB(200,0,255)
paletteRGBs[3] = packRGB(124,12,232)
paletteRGBs[4] = packRGB(70,13,255)

ambientR = 15
ambientG = 0
ambientB = 0

function getRandomVelocity() { return random(4) + 3 }

// init RGB of each laser:
laserRGBs = createArray(laserCount, function(i){ return paletteRGBs[i % numPaletteRGBs] }, true)

// init randomized starting positions of each laser:
laserPositions = createArray(laserCount, function(){ return random(pixelCount) }, true)

// init each laser's velocity
laserVelocities = createArray(laserCount, function(){ return getRandomVelocity() }, true)

// init the full pixel array:
pixelRGBs = createArray(pixelCount)

export function beforeRender(delta) {
  // fade existing pixels:
  for (pixelIndex = 0; pixelIndex < pixelCount; pixelIndex++) {
    pixelRGBs[pixelIndex] = packRGB(
      floor(getR(pixelRGBs[pixelIndex]) * fadeFactor),
      floor(getG(pixelRGBs[pixelIndex]) * fadeFactor),
      floor(getB(pixelRGBs[pixelIndex]) * fadeFactor)
    )
  }

  // advance laser positions:
  for (laserIndex = 0; laserIndex < laserCount; laserIndex++) {
    currentLaserPosition = laserPositions[laserIndex]
    nextLaserPosition = currentLaserPosition + (delta * speedFactor * laserVelocities[laserIndex])
    for (pixelIndex = floor(nextLaserPosition); pixelIndex >= currentLaserPosition; pixelIndex--) {
      // draw new laser edge, but fill in "gaps" from last draw:
      if (pixelIndex < pixelCount) {
        pixelRGBs[pixelIndex] = packRGB(
            min(255, getR(pixelRGBs[pixelIndex]) + getR(laserRGBs[laserIndex])),
            min(255, getG(pixelRGBs[pixelIndex]) + getG(laserRGBs[laserIndex])),
            min(255, getB(pixelRGBs[pixelIndex]) + getB(laserRGBs[laserIndex]))
        )
      }
    }

    laserPositions[laserIndex] = nextLaserPosition
    if (laserPositions[laserIndex] >= pixelCount) {
      // wrap this laser back to the start
      laserPositions[laserIndex] = 0
      laserVelocities[laserIndex] = getRandomVelocity()
    }
  }
}

export function render(rawIndex) {
  index = isForwardDirection ? rawIndex : (pixelCount - rawIndex - 1)
  rgb(
    clamp((getR(pixelRGBs[index]) + ambientR) / 255, 0, 1),
    clamp((getG(pixelRGBs[index]) + ambientG) / 255, 0, 1),
    clamp((getB(pixelRGBs[useBlueLightning ? 0 : index]) + ambientB) / 255, 0, 1)
  )
}

//===== UTILS =====
// ARRAY INIT FUNCTIONS:
function createArray(size, valueOrFn, isFn) {
  arr = array(size)
  if (!valueOrFn) return arr
  for (i = 0; i < size; i++) {
    arr[i] = isFn ? valueOrFn(i) : valueOrFn
  }
  return arr
}
// RGB FUNCTIONS:
// assume each component is an 8-bit "int" (0-255)
function packRGB(r, g, b) { return _packColor(r, g, b) }
function getR(value) { return _getFirstComponent(value) }
function getG(value) { return _getSecondComponent(value) }
function getB(value) { return _getThirdComponent(value) }
// HSV FUNCTIONS:
// assume each component is an 8-bit "int" (0-255)
function packHSV(h, s, v) { return _packColor(h, s, v) }
function getH(value) { return _getFirstComponent(value) }
function getS(value) { return _getSecondComponent(value) }
function getV(value) { return _getThirdComponent(value) }
// "PRIVATE" COLOR FUNCTIONS:
// assume each component is an 8-bit "int" (0-255)
function _packColor(a, b, c) { return (a<<8) + b + (c>>8) }
function _getFirstComponent(value) { return (value>>8) & 0xff } // R or H
function _getSecondComponent(value) { return value & 0xff } // G or S
function _getThirdComponent(value) { return (value<<8) & 0xff } // B or V

