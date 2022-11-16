use core::str;
use embassy_sync::blocking_mutex::Mutex;

use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::*;

use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::task::embassy_sync::EspRawMutex;

use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use esp_idf_sys::{self as sys};
use log::info;

use std::cell::RefCell;
use std::sync::Arc;

type LedPin = esp_idf_hal::gpio::PinDriver<'static, Gpio7, Output>;

static LED_PIN: static_cell::StaticCell<Arc<Mutex<EspRawMutex, RefCell<LedPin>>>> =
    static_cell::StaticCell::new();

sys::esp_app_desc!();

fn main() -> anyhow::Result<()> {
    sys::link_patches();
    esp_idf_hal::task::critical_section::link();

    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Hello, world!");

    let peripherals = Peripherals::take().unwrap();
    let led_pin_handle = LED_PIN.init(Arc::new(Mutex::new(RefCell::new(
        PinDriver::output(peripherals.pins.gpio7).unwrap(),
    ))));
    let led_pin = led_pin_handle.clone();
    led_pin.lock(|led_pin| {
        let mut pin = led_pin.borrow_mut();
        pin.set_low().unwrap();
    });

    let led_pin1 = led_pin_handle.clone();
    let led_pin2 = led_pin_handle.clone();

    let thread0 = std::thread::Builder::new()
        .stack_size(7000)
        .spawn(move || worker(led_pin1, true, "Task1", 500))?;
    let thread1 = std::thread::Builder::new()
        .stack_size(7000)
        .spawn(move || worker(led_pin2, false, "Task2", 1000))?;

    info!("Waiting for worker threads");

    thread0.join().unwrap()?;
    thread1.join().unwrap()?;

    info!("Joined worker threads");

    info!("Done");

    loop {
        // Don't let the idle task starve and trigger warnings from the watchdog.
        FreeRtos::delay_ms(100);
    }
}

fn worker(
    led_pin: Arc<Mutex<EspRawMutex, RefCell<LedPin>>>,
    high: bool,
    log_prefix: &str,
    sleep: u32,
) -> anyhow::Result<()> {
    loop {
        info!("{} Run", log_prefix);
        led_pin.lock(|led_pin| {
            let mut pin = led_pin.borrow_mut();
            if high {
                pin.set_high().unwrap();
            } else {
                pin.set_low().unwrap();
            }
        });

        FreeRtos::delay_ms(sleep);
    }
}
