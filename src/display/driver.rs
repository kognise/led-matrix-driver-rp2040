use core::convert::Infallible;

use defmt::info;
use embedded_hal::digital::v2::OutputPin;
use rp2040_hal::{
    gpio::{bank0, Pin, PinState, PullDownDisabled, PushPullOutput},
    Timer,
};

use super::{spec, Color};

pub type Matrix = [[Color; spec::VIRTUAL_WIDTH]; spec::VIRTUAL_HEIGHT];

type ColorPin<Id> = Pin<Id, PushPullOutput>;
type RowPin<Id> = Pin<Id, PushPullOutput>;
type ClkPin<Id> = Pin<Id, PushPullOutput>;
type LatchPin<Id> = Pin<Id, PushPullOutput>;
type OePin<Id> = Pin<Id, PushPullOutput>;

fn bit_state(bit: u8) -> PinState {
    if bit > 0 {
        PinState::High
    } else {
        PinState::Low
    }
}

pub struct Driver<'a> {
    matrix: Matrix,
    tick_counter: usize,
    timer: &'a Timer,
    r1: ColorPin<bank0::Gpio2>,
    r2: ColorPin<bank0::Gpio3>,
    g1: ColorPin<bank0::Gpio4>,
    g2: ColorPin<bank0::Gpio5>,
    b1: ColorPin<bank0::Gpio6>,
    b2: ColorPin<bank0::Gpio7>,
    a: RowPin<bank0::Gpio10>,
    b: RowPin<bank0::Gpio11>,
    c: RowPin<bank0::Gpio12>,
    d: RowPin<bank0::Gpio13>,
    clk: ClkPin<bank0::Gpio8>,
    latch: LatchPin<bank0::Gpio9>,
    oe: OePin<bank0::Gpio14>, // low = enabled, high = disabled
}

impl<'a> Driver<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn init(
        timer: &'a Timer,
        r1: Pin<bank0::Gpio2, PullDownDisabled>,
        r2: Pin<bank0::Gpio3, PullDownDisabled>,
        g1: Pin<bank0::Gpio4, PullDownDisabled>,
        g2: Pin<bank0::Gpio5, PullDownDisabled>,
        b1: Pin<bank0::Gpio6, PullDownDisabled>,
        b2: Pin<bank0::Gpio7, PullDownDisabled>,
        clk: Pin<bank0::Gpio8, PullDownDisabled>,
        latch: Pin<bank0::Gpio9, PullDownDisabled>,
        a: Pin<bank0::Gpio10, PullDownDisabled>,
        b: Pin<bank0::Gpio11, PullDownDisabled>,
        c: Pin<bank0::Gpio12, PullDownDisabled>,
        d: Pin<bank0::Gpio13, PullDownDisabled>,
        oe: Pin<bank0::Gpio14, PullDownDisabled>,
    ) -> Self {
        Self {
            matrix: [[Color::black(); spec::VIRTUAL_WIDTH]; spec::VIRTUAL_HEIGHT],
            tick_counter: 0,
            timer,
            r1: r1.into_push_pull_output_in_state(PinState::Low),
            r2: r2.into_push_pull_output_in_state(PinState::Low),
            g1: g1.into_push_pull_output_in_state(PinState::Low),
            g2: g2.into_push_pull_output_in_state(PinState::Low),
            b1: b1.into_push_pull_output_in_state(PinState::Low),
            b2: b2.into_push_pull_output_in_state(PinState::Low),
            a: a.into_push_pull_output_in_state(PinState::Low),
            b: b.into_push_pull_output_in_state(PinState::Low),
            c: c.into_push_pull_output_in_state(PinState::Low),
            d: d.into_push_pull_output_in_state(PinState::Low),
            clk: clk.into_push_pull_output_in_state(PinState::Low),
            latch: latch.into_push_pull_output_in_state(PinState::Low),
            oe: oe.into_push_pull_output_in_state(PinState::Low),
        }
    }

    pub fn draw_loop(&mut self, mut render: impl FnMut(&mut Matrix)) {
        let start = self.timer.get_counter();
        loop {
            render(&mut self.matrix);
            self.draw().unwrap();

            if self.tick_counter == 1000 {
                let tick_us =
                    (self.timer.get_counter() - start).to_micros() as usize / self.tick_counter;
                let tick_hz = 1_000_000 / tick_us;
                let cell_hz =
                    tick_hz * spec::PHYSICAL_WIDTH * spec::PHYSICAL_HEIGHT / 2;
                info!(
                    "Tick speed: {} Hz ({} μs) | Cell speed: {} {}",
                    tick_us,
                    tick_hz,
                    if cell_hz > 1_000_000 {
                        cell_hz / 1_000_000
                    } else {
                        cell_hz
                    },
                    if cell_hz > 1_000_000 { "MHz" } else { "Hz" }
                );
            }
        }
    }

    #[inline(always)]
    fn draw(&mut self) -> Result<(), Infallible> {
        const MODULO: usize = spec::COLOR_BITMASK as usize + 1;
        let div = (self.tick_counter % MODULO) as u32;
        self.tick_counter += 1;

        for y_pair in 0..(spec::PHYSICAL_HEIGHT / 2) {
            self.oe.set_low()?;
            self.latch.set_low()?;

            // Draw the row pair
            for x in 0..spec::PHYSICAL_WIDTH {
                let n1 = self.get_color(x, y_pair).hex();
                let n2 = self.get_color(x, y_pair + (spec::PHYSICAL_HEIGHT / 2)).hex();

                const SHIFT: usize = 8 - spec::COLOR_BITMASK.count_ones() as usize;
                let (r1, g1, b1) = (
                    div < n1 >> 16 >> SHIFT,
                    div < (n1 >> 8 >> SHIFT) & spec::COLOR_BITMASK,
                    div < (n1 >> SHIFT) & spec::COLOR_BITMASK,
                );
                let (r2, g2, b2) = (
                    div < n2 >> 16 >> SHIFT,
                    div < (n2 >> 8 >> SHIFT) & spec::COLOR_BITMASK,
                    div < (n2 >> SHIFT) & spec::COLOR_BITMASK,
                );

                self.r1.set_state(r1.into())?;
                self.r2.set_state(r2.into())?;
                self.g1.set_state(g1.into())?;
                self.g2.set_state(g2.into())?;
                self.b1.set_state(b1.into())?;
                self.b2.set_state(b2.into())?;

                self.clk.set_high()?;
                cortex_m::asm::delay(1);
                self.clk.set_low()?;
                cortex_m::asm::delay(1);
            }

            self.oe.set_high()?;
            self.latch.set_high()?;
            cortex_m::asm::delay(1);

            // Select the row pair
            self.a.set_state(bit_state(0b0001 & y_pair as u8))?;
            self.b.set_state(bit_state(0b0010 & y_pair as u8))?;
            self.c.set_state(bit_state(0b0100 & y_pair as u8))?;
            self.d.set_state(bit_state(0b1000 & y_pair as u8))?;
        }

        self.oe.set_high()?;
        Ok(())
    }

    fn get_color(&self, x: usize, y: usize) -> Color {
        let (x, y) = spec::physical_to_virtual(x, y);
        self.matrix[y][x]
    }
}
