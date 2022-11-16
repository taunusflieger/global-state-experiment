use core::str;
use embassy_sync::blocking_mutex::Mutex;
use esp_idf_hal::gpio::*;
use esp_idf_hal::interrupt;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::task::embassy_sync::EspRawMutex;
use esp_idf_hal::task::CriticalSection;
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use esp_idf_sys::{self as sys, esp, EspError};
use log::info;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::sync::Arc;
type LedPin = esp_idf_hal::gpio::Gpio18;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use esp_idf_hal::task::executor::EspExecutor;

static NEOPIXEL_PIN: static_cell::StaticCell<Mutex<EspRawMutex, RefCell<LedPin>>> =
    static_cell::StaticCell::new();

static GLOBAL_TEXT: static_cell::StaticCell<Arc<Mutex<EspRawMutex, RefCell<String>>>> =
    static_cell::StaticCell::new();

static CHANNEL: Channel<EspRawMutex, LedState, 1> = Channel::new();

enum LedState {
    On,
    Off,
}

sys::esp_app_desc!();

fn main() -> Result<(), EspError> {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.

    //NEOPIXEL_PIN.lock(|NEOPIXEL_PIN| NEOPIXEL_PIN.borrow_mut().
    //let x = NEOPIXEL_PIN.get();

    init()?;
    run()
}

fn init() -> Result<(), EspError> {
    sys::link_patches();
    esp_idf_hal::task::critical_section::link();
    esp_idf_svc::timer::embassy_time::driver::link();
    esp_idf_svc::timer::embassy_time::queue::link();

    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Hello, world!");

    let peripherals = Peripherals::take().unwrap();
    NEOPIXEL_PIN.init(Mutex::new(RefCell::new(peripherals.pins.gpio18)));

    let gt_handle = GLOBAL_TEXT.init(Arc::new(Mutex::new(RefCell::new(String::from(
        "Global Text",
    )))));

    let gt = gt_handle.clone();
    gt.lock(|gt| info!("{}", gt.borrow_mut()));

    gt.lock(|gt| gt.swap(&RefCell::new(String::from("hhh"))));

    let gt2 = gt_handle.clone();
    gt2.lock(|gt2| info!("{}", gt2.borrow_mut()));

    let test_pin = Mutex::<EspRawMutex, _>::new(RefCell::new(String::from("Local Text")));

    test_pin.lock(|test_pin| info!("{}", test_pin.borrow_mut()));

    test_pin.lock(|test_pin| test_pin.borrow_mut().clear());
    test_pin.lock(|test_pin| info!("{}", test_pin.borrow_mut()));

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
    let executor = EspExecutor::<16, _>::new();

    let tasks = [executor
        .spawn_local(async move {
            log::info!("starting");
            loop {
                Timer::after(Duration::from_secs(5)).await;
                log::info!("woke up");
            }
        })
        .unwrap()];

    executor.run_tasks(|| true, tasks);
    Ok(())
}
