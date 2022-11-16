use core::str;
use embassy_sync::blocking_mutex::Mutex;

use embassy_time::{Duration, Timer};
//use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::*;

use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::task::embassy_sync::EspRawMutex;
use esp_idf_hal::task::executor::EspExecutor;

use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use esp_idf_sys::{self as sys, esp, EspError};
use log::info;

use std::cell::RefCell;
use std::sync::Arc;

type LedPin = esp_idf_hal::gpio::PinDriver<'static, Gpio7, Output>;

static LED_PIN: static_cell::StaticCell<Arc<Mutex<EspRawMutex, RefCell<LedPin>>>> =
    static_cell::StaticCell::new();

sys::esp_app_desc!();

fn main() -> Result<(), EspError> {
    init()?;
    run()
}

fn init() -> Result<(), EspError> {
    sys::link_patches();
    esp_idf_hal::task::critical_section::link();
    esp_idf_svc::timer::embassy_time::driver::link();
    esp_idf_svc::timer::embassy_time::queue::link();

    esp_idf_svc::log::EspLogger::initialize_default();

    unsafe {
        #[allow(clippy::needless_update)]
        esp!(sys::esp_vfs_eventfd_register(
            &sys::esp_vfs_eventfd_config_t {
                max_fds: 5,
                ..Default::default()
            }
        ))?;
    }

    Ok(())
}

fn run() -> Result<(), EspError> {
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

    let executor = EspExecutor::<16, _>::new();
    let led_pin1 = led_pin_handle.clone();

    let task1 = executor
        .spawn_local(async move {
            loop {
                log::info!("task1");
                Timer::after(Duration::from_secs_floor(2)).await;
                //Timer::after(Duration::from_secs(2)).await;
                //embassy_time::Timer::after(embassy_time::Duration::from_millis(500)).await;
                //FreeRtos::delay_ms(500);
                led_pin1.lock(|led_pin1| {
                    let mut pin = led_pin1.borrow_mut();
                    pin.set_high().unwrap();
                });
            }
        })
        .unwrap();

    let led_pin2 = led_pin_handle.clone();
    let task2 = executor
        .spawn_local(async move {
            loop {
                log::info!("task2");
                Timer::after(Duration::from_secs_floor(4)).await;
                //Timer::after(Duration::from_secs(4)).await;
                //embassy_time::Timer::after(embassy_time::Duration::from_millis(1000)).await;
                //FreeRtos::delay_ms(1000);
                led_pin2.lock(|led_pin2| {
                    let mut pin = led_pin2.borrow_mut();
                    pin.set_low().unwrap();
                });
            }
        })
        .unwrap();

    executor.run_tasks(|| true, [task1, task2]);

    Ok(())
}
