use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::*;
use esp_idf_hal::into_ref;

struct LedDriver<'d> {
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
        let green: PinDriver<AnyOutputPin, Output> = PinDriver::output(green_pin.map_into()).unwrap();
        let blue: PinDriver<AnyOutputPin, Output> = PinDriver::output(blue_pin.map_into()).unwrap();
        Self{
            red,
            green,
            blue,
        }
    }

    pub fn set_red(&mut self) {
        self.red.set_high().unwrap();
        self.green.set_low().unwrap();
        self.blue.set_low().unwrap();
    }

    pub fn set_yellow(&mut self) {
        self.red.set_high().unwrap();
        self.green.set_high().unwrap();
        self.blue.set_low().unwrap();
    }

    pub fn set_green(&mut self) {
        self.red.set_low().unwrap();
        self.green.set_high().unwrap();
        self.blue.set_low().unwrap();
    }
}

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    //let mut red = PinDriver::output(peripherals.pins.gpio3).unwrap();
    //let mut green = PinDriver::output(peripherals.pins.gpio4).unwrap();
    //let mut blue = PinDriver::output(peripherals.pins.gpio5).unwrap();
    let mut led_driver= LedDriver::new(
        peripherals.pins.gpio3,
        peripherals.pins.gpio4,
        peripherals.pins.gpio5);

    loop {
        led_driver.set_red();
        FreeRtos::delay_ms(3000);
        led_driver.set_yellow();
        FreeRtos::delay_ms(1000);
        led_driver.set_green();
        FreeRtos::delay_ms(3000);
        led_driver.set_yellow();
        FreeRtos::delay_ms(1000);
    }
}
