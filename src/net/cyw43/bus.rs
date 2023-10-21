use core::{fmt::Debug, slice};

// use embassy_time::{Duration, Timer};
use embedded_hal::digital::v2::OutputPin;
// use embedded_hal_async::spi::{transaction, SpiBusRead, SpiBusWrite, SpiDevice};

use crate::net::cyw43::consts::*;

pub(crate) struct Bus<PWR, SPI> {
    backplane_window: u32,
    pwr: PWR,
    spi: SPI,
}

impl<PWR, SPI> Bus<PWR, SPI>
where
    PWR: OutputPin,
    PWR::Error: Debug,
    SPI: SpiDevice,
    SPI::Bus: SpiBusRead<u32> + SpiBusWrite<u32>,
{
    pub(crate) fn new(pwr: PWR, spi: SPI) -> Self {
        Self {
            backplane_window: 0xAAAA_AAAA,
            pwr,
            spi,
        }
    }

    pub async fn init(&mut self) {
        // Reset
        self.pwr.set_low().unwrap();
        Timer::after(Duration::from_millis(20)).await;
        self.pwr.set_high().unwrap();
        Timer::after(Duration::from_millis(250)).await;

        while self.read32_swapped(REG_BUS_TEST_RO).await != FEEDBEAD {}

        self.write32_swapped(REG_BUS_TEST_RW, TEST_PATTERN).await;
        let val = self.read32_swapped(REG_BUS_TEST_RW).await;
        assert_eq!(val, TEST_PATTERN);

        // 32-bit word length, little endian (which is the default endianess).
        self.write32_swapped(REG_BUS_CTRL, WORD_LENGTH_32 | HIGH_SPEED)
            .await;

        let val = self.read32(FUNC_BUS, REG_BUS_TEST_RO).await;
        assert_eq!(val, FEEDBEAD);
        let val = self.read32(FUNC_BUS, REG_BUS_TEST_RW).await;
        assert_eq!(val, TEST_PATTERN);
    }

    pub async fn wlan_read(&mut self, buf: &mut [u32], len_in_u8: u32) {
        let cmd = cmd_word(READ, INC_ADDR, FUNC_WLAN, 0, len_in_u8);
        let len_in_u32 = (len_in_u8 as usize + 3) / 4;
        transaction!(&mut self.spi, |bus| async {
            bus.write(&[cmd]).await?;
            bus.read(&mut buf[..len_in_u32]).await?;
            Ok(())
        })
        .await
        .unwrap();
    }

    pub async fn wlan_write(&mut self, buf: &[u32]) {
        let cmd = cmd_word(WRITE, INC_ADDR, FUNC_WLAN, 0, buf.len() as u32 * 4);
        transaction!(&mut self.spi, |bus| async {
            bus.write(&[cmd]).await?;
            bus.write(buf).await?;
            Ok(())
        })
        .await
        .unwrap();
    }

    #[allow(unused)]
    pub async fn bp_read(&mut self, mut addr: u32, mut data: &mut [u8]) {
        // It seems the HW force-aligns the addr
        // to 2 if data.len() >= 2
        // to 4 if data.len() >= 4
        // To simplify, enforce 4-align for now.
        assert!(addr % 4 == 0);

        let mut buf = [0u32; BACKPLANE_MAX_TRANSFER_SIZE / 4];

        while !data.is_empty() {
            // Ensure transfer doesn't cross a window boundary.
            let window_offs = addr & BACKPLANE_ADDRESS_MASK;
            let window_remaining = BACKPLANE_WINDOW_SIZE - window_offs as usize;

            let len = data
                .len()
                .min(BACKPLANE_MAX_TRANSFER_SIZE)
                .min(window_remaining);

            self.backplane_set_window(addr).await;

            let cmd = cmd_word(READ, INC_ADDR, FUNC_BACKPLANE, window_offs, len as u32);

            transaction!(&mut self.spi, |bus| async {
                bus.write(&[cmd]).await?;

                // 4-byte response delay.
                let mut junk = [0; 1];
                bus.read(&mut junk).await?;

                // Read data
                bus.read(&mut buf[..(len + 3) / 4]).await?;
                Ok(())
            })
            .await
            .unwrap();

            data[..len].copy_from_slice(&slice8_mut(&mut buf)[..len]);

            // Advance ptr.
            addr += len as u32;
            data = &mut data[len..];
        }
    }

    pub async fn bp_write(&mut self, mut addr: u32, mut data: &[u8]) {
        // It seems the HW force-aligns the addr
        // to 2 if data.len() >= 2
        // to 4 if data.len() >= 4
        // To simplify, enforce 4-align for now.
        assert!(addr % 4 == 0);

        let mut buf = [0u32; BACKPLANE_MAX_TRANSFER_SIZE / 4];

        while !data.is_empty() {
            // Ensure transfer doesn't cross a window boundary.
            let window_offs = addr & BACKPLANE_ADDRESS_MASK;
            let window_remaining = BACKPLANE_WINDOW_SIZE - window_offs as usize;

            let len = data
                .len()
                .min(BACKPLANE_MAX_TRANSFER_SIZE)
                .min(window_remaining);
            slice8_mut(&mut buf)[..len].copy_from_slice(&data[..len]);

            self.backplane_set_window(addr).await;

            let cmd = cmd_word(WRITE, INC_ADDR, FUNC_BACKPLANE, window_offs, len as u32);

            transaction!(&mut self.spi, |bus| async {
                bus.write(&[cmd]).await?;
                bus.write(&buf[..(len + 3) / 4]).await?;
                Ok(())
            })
            .await
            .unwrap();

            // Advance ptr.
            addr += len as u32;
            data = &data[len..];
        }
    }

    pub async fn bp_read8(&mut self, addr: u32) -> u8 {
        self.backplane_readn(addr, 1).await as u8
    }

    pub async fn bp_write8(&mut self, addr: u32, val: u8) {
        self.backplane_writen(addr, val as u32, 1).await
    }

    pub async fn bp_read16(&mut self, addr: u32) -> u16 {
        self.backplane_readn(addr, 2).await as u16
    }

    #[allow(unused)]
    pub async fn bp_write16(&mut self, addr: u32, val: u16) {
        self.backplane_writen(addr, val as u32, 2).await
    }

    #[allow(unused)]
    pub async fn bp_read32(&mut self, addr: u32) -> u32 {
        self.backplane_readn(addr, 4).await
    }

    pub async fn bp_write32(&mut self, addr: u32, val: u32) {
        self.backplane_writen(addr, val, 4).await
    }

    async fn backplane_readn(&mut self, addr: u32, len: u32) -> u32 {
        self.backplane_set_window(addr).await;

        let mut bus_addr = addr & BACKPLANE_ADDRESS_MASK;
        if len == 4 {
            bus_addr |= BACKPLANE_ADDRESS_32BIT_FLAG
        }
        self.readn(FUNC_BACKPLANE, bus_addr, len).await
    }

    async fn backplane_writen(&mut self, addr: u32, val: u32, len: u32) {
        self.backplane_set_window(addr).await;

        let mut bus_addr = addr & BACKPLANE_ADDRESS_MASK;
        if len == 4 {
            bus_addr |= BACKPLANE_ADDRESS_32BIT_FLAG
        }
        self.writen(FUNC_BACKPLANE, bus_addr, val, len).await
    }

    async fn backplane_set_window(&mut self, addr: u32) {
        let new_window = addr & !BACKPLANE_ADDRESS_MASK;

        if (new_window >> 24) as u8 != (self.backplane_window >> 24) as u8 {
            self.write8(
                FUNC_BACKPLANE,
                REG_BACKPLANE_BACKPLANE_ADDRESS_HIGH,
                (new_window >> 24) as u8,
            )
            .await;
        }
        if (new_window >> 16) as u8 != (self.backplane_window >> 16) as u8 {
            self.write8(
                FUNC_BACKPLANE,
                REG_BACKPLANE_BACKPLANE_ADDRESS_MID,
                (new_window >> 16) as u8,
            )
            .await;
        }
        if (new_window >> 8) as u8 != (self.backplane_window >> 8) as u8 {
            self.write8(
                FUNC_BACKPLANE,
                REG_BACKPLANE_BACKPLANE_ADDRESS_LOW,
                (new_window >> 8) as u8,
            )
            .await;
        }
        self.backplane_window = new_window;
    }

    pub async fn read8(&mut self, func: u32, addr: u32) -> u8 {
        self.readn(func, addr, 1).await as u8
    }

    pub async fn write8(&mut self, func: u32, addr: u32, val: u8) {
        self.writen(func, addr, val as u32, 1).await
    }

    pub async fn read16(&mut self, func: u32, addr: u32) -> u16 {
        self.readn(func, addr, 2).await as u16
    }

    #[allow(unused)]
    pub async fn write16(&mut self, func: u32, addr: u32, val: u16) {
        self.writen(func, addr, val as u32, 2).await
    }

    pub async fn read32(&mut self, func: u32, addr: u32) -> u32 {
        self.readn(func, addr, 4).await
    }

    #[allow(unused)]
    pub async fn write32(&mut self, func: u32, addr: u32, val: u32) {
        self.writen(func, addr, val, 4).await
    }

    async fn readn(&mut self, func: u32, addr: u32, len: u32) -> u32 {
        let cmd = cmd_word(READ, INC_ADDR, func, addr, len);
        let mut buf = [0; 1];

        transaction!(&mut self.spi, |bus| async {
            bus.write(&[cmd]).await?;
            if func == FUNC_BACKPLANE {
                // 4-byte response delay.
                bus.read(&mut buf).await?;
            }
            bus.read(&mut buf).await?;
            Ok(())
        })
        .await
        .unwrap();

        buf[0]
    }

    async fn writen(&mut self, func: u32, addr: u32, val: u32, len: u32) {
        let cmd = cmd_word(WRITE, INC_ADDR, func, addr, len);

        transaction!(&mut self.spi, |bus| async {
            bus.write(&[cmd, val]).await?;
            Ok(())
        })
        .await
        .unwrap();
    }

    async fn read32_swapped(&mut self, addr: u32) -> u32 {
        let cmd = cmd_word(READ, INC_ADDR, FUNC_BUS, addr, 4);
        let mut buf = [0; 1];

        transaction!(&mut self.spi, |bus| async {
            bus.write(&[swap16(cmd)]).await?;
            bus.read(&mut buf).await?;
            Ok(())
        })
        .await
        .unwrap();

        swap16(buf[0])
    }

    async fn write32_swapped(&mut self, addr: u32, val: u32) {
        let cmd = cmd_word(WRITE, INC_ADDR, FUNC_BUS, addr, 4);

        transaction!(&mut self.spi, |bus| async {
            bus.write(&[swap16(cmd), swap16(val)]).await?;
            Ok(())
        })
        .await
        .unwrap();
    }
}

fn swap16(x: u32) -> u32 {
    x.rotate_left(16)
}

fn cmd_word(write: bool, incr: bool, func: u32, addr: u32, len: u32) -> u32 {
    (write as u32) << 31
        | (incr as u32) << 30
        | (func & 0b11) << 28
        | (addr & 0x1FFFF) << 11
        | (len & 0x7FF)
}

fn slice8_mut(x: &mut [u32]) -> &mut [u8] {
    let len = x.len() * 4;
    unsafe { slice::from_raw_parts_mut(x.as_mut_ptr() as _, len) }
}
