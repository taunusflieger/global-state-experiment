# global-state-experiment


This project helped me to answer the question how to share a GPIO pin across different threads. There are two branches:
* **FreeRTOS-Threads** here the threads are created using EP-IDF FreeRTOS
* **master** here the threads are created using embassy-sync. This version doesn't work as expected as the Timer function doesn't behave like expected. It looks like that after a while the timer doesn't create the specified delay.

## How does the sharing work?
In the main thread the shared GPIO pin is stored in 
```rust
type LedPin = esp_idf_hal::gpio::PinDriver<'static, Gpio7, Output>;

static LED_PIN: static_cell::StaticCell<Arc<Mutex<EspRawMutex, RefCell<LedPin>>>> = 
    static_cell::StaticCell::new();
```
which provides a thread safe way to handle access to the shared GPIO pin

## How to build?
The project is targeting the [`esp-rust-board`](https://github.com/esp-rs/esp-rust-board) which uses a esp32c3 SoC. The led pin used is GPIO7.

```sh
cargo espflash flash --monitor --port /dev/ttyACM0  
```

The project creates two threads. One thread sets the led pin to high, the other thread sets the pin to low. Both threads have different delay/sleep values. The result is a blinking led.
