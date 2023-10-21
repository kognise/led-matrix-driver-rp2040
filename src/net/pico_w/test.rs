// use core::mem::transmute;

// use defmt::{info, println};
// use embedded_hal::digital::v2::PinState;
// use embedded_hal_async::spi::ExclusiveDevice;
// use rp2040_hal::gpio::bank0::{Gpio23, Gpio25};
// use rp2040_hal::gpio::{Output, Pin, Pins, PushPull};

// use crate::legacy_pin::LegacyPin;
// use crate::net::cyw43;

// use super::spi_bus::SpiBus;

// // type Power = Pin<Gpio23, Output<PushPull>>;
// // type Spi = ExclusiveDevice<SpiBus, LegacyPin<Gpio25, Output<PushPull>>>;

// #[embassy_executor::task]
// async fn wifi_task(runner: cyw43::Runner<'static, Power, Spi>) -> ! {
//     runner.run().await
// }

// pub async fn connect_test(_pins: Pins, _spawner: embassy_executor::Spawner) -> ! {
//     // let power: Power = pins
//     //     .gpio23
//     //     .into_push_pull_output_in_state(PinState::Low);
//     // let chip_select = pins.gpio25.into_push_pull_output_in_state(PinState::High);
//     // let clk = pins
//     //     .gpio29
//     //     .into_push_pull_output_in_state(PinState::Low);
//     // let dio = pins
//     //     .gpio24
//     //     .into_readable_output_in_state(PinState::Low);

//     // let bus = SpiBus::new(clk, dio);
//     // let spi: Spi = ExclusiveDevice::new(bus, LegacyPin::from_pin(chip_select));

//     // let mut state = cyw43::State::default();

//     // let (net_device, mut control, runner) = {
//     //     // SAFETY: Function returns never type, so guaranteed to be static.
//     //     let static_state_ref: &'static mut cyw43::State = unsafe { transmute(&mut state) };
//     //     cyw43::new(static_state_ref, power, spi)
//     // }
//     // .await;

//     // spawner.spawn(wifi_task(runner)).unwrap();

//     // control.init().await;
//     // control
//     //     .set_power_management(cyw43::PowerManagementMode::PowerSave)
//     //     .await;

//     println!("hi");
//     info!("Connecting to Wi-Fi...");
//     // control.join_wpa2("test", "bettertobeapiratethanjointhenavy").await;
//     info!("Connected to Wi-Fi!");

//     #[allow(clippy::empty_loop)]
//     loop {}
// }
