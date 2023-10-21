// // Fun fact, embedded-hal-async imports an old version of embedded-hal
// // with a different API for OutputPin. Because I'm not going to fork
// // embedded-hal-async, I need to make this disgusting compatibility
// // layer instead.

// use core::convert::Infallible;
// use embedded_hal::digital::v2::OutputPin;
// // use embedded_hal_legacy::digital::ErrorType as LegacyErrorType;
// // use embedded_hal_legacy::digital::OutputPin as LegacyOutputPin;
// use rp2040_hal::gpio::{Output, OutputConfig, Pin, PinId, PinMode, ValidPinMode};

// pub struct LegacyPin<I, M>(Pin<I, M>)
// where
//     I: PinId,
//     M: PinMode + ValidPinMode<I>;

// impl<I, M> LegacyPin<I, M>
// where
//     I: PinId,
//     M: PinMode + ValidPinMode<I>,
// {
//     pub fn from_pin(pin: Pin<I, M>) -> Self {
//         Self(pin)
//     }
// }

// // impl<I, M> LegacyErrorType for LegacyPin<I, M>
// // where
// //     I: PinId,
// //     M: PinMode + ValidPinMode<I>,
// // {
// //     type Error = Infallible;
// // }

// // impl<I, C> LegacyOutputPin for LegacyPin<I, Output<C>>
// // where
// //     I: PinId,
// //     C: OutputConfig,
// // {
// //     #[inline]
// //     fn set_high(&mut self) -> Result<(), Self::Error> {
// //         self.0.set_high()
// //     }

// //     #[inline]
// //     fn set_low(&mut self) -> Result<(), Self::Error> {
// //         self.0.set_low()
// //     }
// // }
