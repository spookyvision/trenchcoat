# Trenchcoat changelog

## 0.5

### 0.5.0

- support `no_std` with `alloc` enabled
- remove stale workspace and make stm32f4 app compile again, ahem

## 0.4

### 0.4.5
- esp32 app
- web app: add endpoint selection

### 0.4.4
- fixed another embarassing mistake in web app (input cursor jerk)

### 0.4.3
- fix CORS workaround in python webserver
- highly improved web app code sanity (much less state updates/clones)

### 0.4.2
- documentation

### 0.4.1
- add python web server to bridge http to UART, improve web app

### 0.4.0
- opt-in alloc/`std::collections` support

## 0.3
- simplified public compiler api
## 0.2
- hot code replacement
## 0.1 
- initial release
