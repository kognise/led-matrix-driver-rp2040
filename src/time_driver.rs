use core::cell::Cell;

use cortex_m_rt::interrupt;
use critical_section::{Mutex, CriticalSection};
// use atomic_polyfill::{AtomicU8, Ordering};
// use critical_section::CriticalSection;
// use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
// use embassy_sync::blocking_mutex::Mutex;
use embassy_time::driver::{AlarmHandle, Driver};
use rp2040_hal::timer::Alarm;
use rp2040_hal::pac::TIMER;

// use crate::interrupt::{Interrupt, InterruptExt};
// use crate::{interrupt, pac};

#[allow(clippy::type_complexity)]
struct AlarmState {
    timestamp: Cell<u64>,
    callback: Cell<Option<(fn(*mut ()), *mut ())>>,
}
unsafe impl Send for AlarmState {}

const ALARM_COUNT: usize = 4;
#[allow(clippy::declare_interior_mutable_const)]
const DUMMY_ALARM: AlarmState = AlarmState {
    timestamp: Cell::new(0),
    callback: Cell::new(None),
};

struct TimerDriver {
    alarms: Mutex<[AlarmState; ALARM_COUNT]>,
    next_alarm: Mutex<u8>,
}

embassy_time::time_driver_impl!(
    static DRIVER: TimerDriver = TimerDriver {
        alarms: Mutex::new([DUMMY_ALARM; ALARM_COUNT]),
        next_alarm: Mutex::new(0),
    }
);

unsafe fn timer() -> &'static rp2040_hal::pac::timer::RegisterBlock {
	TIMER::ptr().as_ref().unwrap()
}

impl Driver for TimerDriver {
    fn now(&self) -> u64 {
        loop {
            unsafe {
                let hi = timer().timerawh.read().bits();
                let lo = timer().timerawl.read().bits();
                let hi2 = timer().timerawh.read().bits();
                if hi == hi2 {
                    return (hi as u64) << 32 | (lo as u64);
                }
            }
        }
    }

    unsafe fn allocate_alarm(&self) -> Option<AlarmHandle> {
        let id = critical_section::with(|cs| {
            let next_alarm = *self.next_alarm.borrow(cs);
            if next_alarm < ALARM_COUNT as u8 {
                Some(next_alarm + 1)
            } else {
                None
            }
        });
        id.map(|id| AlarmHandle::new(id))
    }

    fn set_alarm_callback(&self, alarm: AlarmHandle, callback: fn(*mut ()), ctx: *mut ()) {
        let n = alarm.id() as usize;
        critical_section::with(|cs| {
            let alarm = &self.alarms.borrow(cs)[n];
            alarm.callback.set(Some((callback, ctx)));
        })
    }

    fn set_alarm(&self, alarm: AlarmHandle, timestamp: u64) -> bool {
        let n = alarm.id() as usize;
        critical_section::with(|cs| {
            let alarm = &self.alarms.borrow(cs)[n];
            alarm.timestamp.set(timestamp);

            // Arm it.
            // Note that we're not checking the high bits at all. This means the irq may fire early
            // if the alarm is more than 72 minutes (2^32 us) in the future. This is OK, since on irq fire
            // it is checked if the alarm time has passed.
            unsafe {
                match n {
                    0 => timer().alarm0.write(|w| w.bits(timestamp as u32)),
                    1 => timer().alarm1.write(|w| w.bits(timestamp as u32)),
                    2 => timer().alarm2.write(|w| w.bits(timestamp as u32)),
                    3 => timer().alarm3.write(|w| w.bits(timestamp as u32)),
                    _ => unreachable!(),
                }
            };

            let now = self.now();
            if timestamp <= now {
                // If alarm timestamp has passed the alarm will not fire.
                // Disarm the alarm and return `false` to indicate that.
                unsafe { timer().armed.write(|w| w.armed().bits(1 << n)) }

                alarm.timestamp.set(u64::MAX);

                false
            } else {
                true
            }
        })
    }
}

impl TimerDriver {
    fn check_alarm(&self, n: usize) {
        critical_section::with(|cs| {
            let alarm = &self.alarms.borrow(cs)[n];
            let timestamp = alarm.timestamp.get();
            if timestamp <= self.now() {
                self.trigger_alarm(n, cs)
            } else {
                // Not elapsed, arm it again.
                // This can happen if it was set more than 2^32 us in the future.
                unsafe {
                    match n {
                        0 => timer().alarm0.write(|w| w.bits(timestamp as u32)),
                        1 => timer().alarm1.write(|w| w.bits(timestamp as u32)),
                        2 => timer().alarm2.write(|w| w.bits(timestamp as u32)),
                        3 => timer().alarm3.write(|w| w.bits(timestamp as u32)),
                        _ => unreachable!(),
                    }
                };
            }
        });

        // clear the irq
        unsafe {
            timer().intr.write(|w| match n {
                0 => w.alarm_0().set_bit(),
                1 => w.alarm_1().set_bit(),
                2 => w.alarm_2().set_bit(),
                3 => w.alarm_3().set_bit(),
                _ => unreachable!(),
            })
        }
    }

    fn trigger_alarm(&self, n: usize, cs: CriticalSection) {
        // disarm
        unsafe { timer().armed.write(|w| w.armed().bits(1 << n)) }

        let alarm = &self.alarms.borrow(cs)[n];
        alarm.timestamp.set(u64::MAX);

        // Call after clearing alarm, so the callback can set another alarm.
        if let Some((f, ctx)) = alarm.callback.get() {
            f(ctx);
        }
    }
}

/// ## Safety
/// 
/// Must be called exactly once at bootup.
pub unsafe fn init(timer_ctrl: &mut rp2040_hal::Timer) {
    // init alarms
    critical_section::with(|cs| {
        let alarms = DRIVER.alarms.borrow(cs);
        for a in alarms {
            a.timestamp.set(u64::MAX);
        }
    });

    // enable all irqs
    timer().inte.write(|w| {
        let w = w.alarm_0().set_bit();
		let w = w.alarm_1().set_bit();
		let w = w.alarm_2().set_bit();
		let w = w.alarm_3().set_bit();
		w
    });

    timer_ctrl.alarm_0().unwrap().enable_interrupt();
    timer_ctrl.alarm_1().unwrap().enable_interrupt();
    timer_ctrl.alarm_2().unwrap().enable_interrupt();
    timer_ctrl.alarm_3().unwrap().enable_interrupt();
}

// #[rp_pico::pac::interrupt]
// unsafe fn TIMER_IRQ_0() {
//     DRIVER.check_alarm(0)
// }

// #[rp2040_hal::pac::interrupt]
// unsafe fn TIMER_IRQ_1() {
//     DRIVER.check_alarm(1)
// }

#[interrupt]
unsafe fn TIMER_IRQ_2() {
    DRIVER.check_alarm(2)
}

// #[interrupt]
// unsafe fn TIMER_IRQ_3() {
//     DRIVER.check_alarm(3)
// }