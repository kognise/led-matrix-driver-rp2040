#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::mem::transmute;

use cortex_m::delay::Delay;
// use board::{display, dvd_logo};
use defmt::println;
use defmt_rtt as _;
// use fugit::TimerDurationU64;
use panic_probe as _;
// use rp2040_hal::timer::Instant;
use rp2040_hal::{clocks, entry, gpio, pac, Sio, Timer, Watchdog};

const XOSC_FREQ_HZ: u32 = 12_000_000;
// const TIMER_HZ: u32 = 1_000_000;

#[embassy_executor::task]
async fn main(_spawner: embassy_executor::Spawner) {
    println!("Hello, world!");

    let mut pac = pac::Peripherals::take().unwrap();
    let sio = Sio::new(pac.SIO);
    let pins = gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // configure clocks
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let _clocks = clocks::init_clocks_and_plls(
        XOSC_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let _timer = Timer::new(pac.TIMER, &mut pac.RESETS);

    // Display rendering function.
    const LOGO_REFRESH_MS: u64 = 60;
    const LOGO_REFRESH_DURATION: TimerDurationU64<TIMER_HZ> =
        TimerDurationU64::from_ticks(LOGO_REFRESH_MS * 1000);
    let colors = [
        0xffffff, 0xff0000, 0x00ff00, 0x0000ff, 0xffff00, 0x00ffff, 0xff00ff,
    ].map(display::Color::from_hex);
    let mut last_update = Instant::from_ticks(0);
    let (mut x, mut y) = (0, 10);
    let (mut dx, mut dy) = (1_isize, 1_isize);
    let mut color_index = 0;

    let render = |matrix: &mut display::Matrix| {
        if timer.get_counter() - last_update >= LOGO_REFRESH_DURATION {
            last_update = timer.get_counter();
            x = (x as isize + dx) as usize;
            y = (y as isize + dy) as usize;

            *matrix = [[display::Color::black(); display::spec::VIRTUAL_WIDTH]; display::spec::VIRTUAL_HEIGHT];
            let dvd_logo = dvd_logo::make_dvd_logo(colors[color_index % colors.len()]);
            for (dvd_y, row) in dvd_logo.iter().enumerate() {
                for (dvd_x, cell) in row.iter().enumerate() {
                    matrix[dvd_y + y][dvd_x + x] = *cell;
                }
            }

            let mut bounced = false;
            if x == 0 || x == display::spec::VIRTUAL_WIDTH - dvd_logo::WIDTH {
                dx = -dx;
                bounced = true;
            }
            if y == 0 || y == display::spec::VIRTUAL_HEIGHT - dvd_logo::HEIGHT {
                dy = -dy;
                bounced = true;
            }
            if bounced {
                color_index += 1;
            }
        }
    };

    // Initialize display and run draw loop.
    let mut display = display::Driver::init(
        &timer,
        pins.gpio2,
        pins.gpio3,
        pins.gpio4,
        pins.gpio5,
        pins.gpio6,
        pins.gpio7,
        pins.gpio8,
        pins.gpio9,
        pins.gpio10,
        pins.gpio11,
        pins.gpio12,
        pins.gpio13,
        pins.gpio14,
    );
    display.draw_loop(render);
}

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

#[entry]
fn entry() -> ! {
    // SAFETY: Entrypoint returns never type, so guaranteed to be static.
    let executor: &'static mut embassy_executor::Executor =
        unsafe { transmute(&mut embassy_executor::Executor::new()) };
    executor.run(|spawner| spawner.must_spawn(main(spawner)));
}
