use core::convert::Infallible;

use embedded_hal::digital::v2::{InputPin, OutputPin};
use embedded_hal_async::spi::{ErrorType, SpiBusFlush, SpiBusRead, SpiBusWrite};
use rp2040_hal::gpio::bank0::{Gpio24, Gpio29};
use rp2040_hal::gpio::{Pin, PushPullOutput, ReadableOutput};

pub struct SpiBus {
    /// SPI clock
    clk: Pin<Gpio29, PushPullOutput>,

    /// 4 signals, all in one!! Pin 24
    /// - SPI MISO
    /// - SPI MOSI
    /// - IRQ
    /// - strap to set to gSPI mode on boot.
    dio: Pin<Gpio24, ReadableOutput>,
}

impl SpiBus {
    pub fn new(clk: Pin<Gpio29, PushPullOutput>, dio: Pin<Gpio24, ReadableOutput>) -> Self {
        Self { clk, dio }
    }
}

impl ErrorType for SpiBus {
    type Error = Infallible;
}

impl SpiBusFlush for SpiBus {
    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl SpiBusRead<u32> for SpiBus {
    async fn read(&mut self, words: &mut [u32]) -> Result<(), Self::Error> {
        // self.dio.set_as_input();
        for word in words {
            let mut w = 0;
            for _ in 0..32 {
                w <<= 1;

                // rising edge, sample data
                if self.dio.is_high()? {
                    w |= 0x01;
                }
                self.clk.set_high()?;

                // falling edge
                self.clk.set_low()?;
            }
            *word = w
        }

        Ok(())
    }
}

impl SpiBusWrite<u32> for SpiBus {
    async fn write(&mut self, words: &[u32]) -> Result<(), Self::Error> {
        // self.dio.set_as_output();
        for word in words {
            let mut word = *word;
            for _ in 0..32 {
                // falling edge, setup data
                self.clk.set_low()?;
                if word & 0x8000_0000 == 0 {
                    self.dio.set_low()?;
                } else {
                    self.dio.set_high()?;
                }

                // rising edge
                self.clk.set_high()?;

                word <<= 1;
            }
        }
        self.clk.set_low()?;

        // self.dio.set_as_input();
        Ok(())
    }
}
