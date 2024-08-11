use esp_idf_hal::gpio::*;
use esp_idf_hal::into_ref;
use esp_idf_hal::peripheral::Peripheral;

pub struct LedDriver<'d> {
    red: PinDriver<'d, AnyOutputPin, Output>,
    green: PinDriver<'d, AnyOutputPin, Output>,
    blue: PinDriver<'d, AnyOutputPin, Output>,
}

impl<'d> LedDriver<'d> {
    pub fn new(
        red_pin: impl Peripheral<P = impl OutputPin> + 'd,
        green_pin: impl Peripheral<P = impl OutputPin> + 'd,
        blue_pin: impl Peripheral<P = impl OutputPin> + 'd,
    ) -> Self {
        into_ref!(red_pin);
        into_ref!(green_pin);
        into_ref!(blue_pin);
        let red: PinDriver<AnyOutputPin, Output> = PinDriver::output(red_pin.map_into()).unwrap();
        let green: PinDriver<AnyOutputPin, Output> =
            PinDriver::output(green_pin.map_into()).unwrap();
        let blue: PinDriver<AnyOutputPin, Output> = PinDriver::output(blue_pin.map_into()).unwrap();
        Self { red, green, blue }
    }

    pub fn set_leds(&mut self, red: bool, green: bool, blue: bool) {
        self.red.set_level(red.into()).unwrap();
        self.green.set_level(green.into()).unwrap();
        self.blue.set_level(blue.into()).unwrap();
    }
}
