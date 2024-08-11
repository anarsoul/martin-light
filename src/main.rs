use embedded_svc::mqtt::client::{
    Details::Complete, EventPayload::Error, EventPayload::Received, QoS,
};
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::*;
use esp_idf_hal::into_ref;
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::mqtt::client::{Details, EspMqttClient, MqttClientConfiguration};
use log::{info, warn};
use std::str;
use std::sync::mpsc;
use wifi::wifi;

#[toml_cfg::toml_config]
pub struct Config {
    #[default("mqttserver")]
    mqtt_host: &'static str,
    #[default("")]
    mqtt_user: &'static str,
    #[default("")]
    mqtt_pass: &'static str,
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
}

struct LedDriver<'d> {
    red: PinDriver<'d, AnyOutputPin, Output>,
    green: PinDriver<'d, AnyOutputPin, Output>,
    blue: PinDriver<'d, AnyOutputPin, Output>,
}

struct Color {
    red: bool,
    green: bool,
    blue: bool,
    duration: i32,
}

enum Mode {
    Cycle,
    Red,
    Yellow,
    Green,
    Blue,
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

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take().unwrap();

    let app_config = CONFIG;

    let _wifi = wifi(
        app_config.wifi_ssid,
        app_config.wifi_psk,
        peripherals.modem,
        sysloop,
    )
    .unwrap();

    let mqtt_config = MqttClientConfiguration::default();

    let broker_url = if !app_config.mqtt_user.is_empty() {
        format!(
            "mqtt://{}:{}@{}",
            app_config.mqtt_user, app_config.mqtt_pass, app_config.mqtt_host
        )
    } else {
        format!("mqtt://{}", app_config.mqtt_host)
    };

    info!("Broker URL: {}", broker_url);

    let mut led_driver = LedDriver::new(
        peripherals.pins.gpio3,
        peripherals.pins.gpio4,
        peripherals.pins.gpio5,
    );

    let (tx, rx) = mpsc::channel::<Mode>();
    let mut client =
        EspMqttClient::new_cb(
            &broker_url,
            &mqtt_config,
            move |message_event| match message_event.payload() {
                Received { data, details, .. } => process_message(data, details, &tx),
                Error(e) => warn!("Received error from MQTT: {:?}", e),
                _ => info!("Received from MQTT: {:?}", message_event.payload()),
            },
        )
        .unwrap();

    // Yeah, a dirty hack. We need to give it some time to connect
    FreeRtos::delay_ms(1000);
    client
        .subscribe("esp32/martin_light", QoS::AtLeastOnce)
        .unwrap();

    let cycle_seq: Vec<Color> = vec![
        Color {
            red: true,
            green: false,
            blue: false,
            duration: 3000,
        },
        Color {
            red: true,
            green: true,
            blue: false,
            duration: 1000,
        },
        Color {
            red: false,
            green: true,
            blue: false,
            duration: 3000,
        },
        Color {
            red: true,
            green: true,
            blue: false,
            duration: 1000,
        },
    ];
    let red_seq: Vec<Color> = vec![Color {
        red: true,
        green: false,
        blue: false,
        duration: 1000,
    }];
    let green_seq: Vec<Color> = vec![Color {
        red: false,
        green: true,
        blue: false,
        duration: 1000,
    }];
    let yellow_seq: Vec<Color> = vec![Color {
        red: true,
        green: true,
        blue: false,
        duration: 3000,
    }];
    let blue_seq: Vec<Color> = vec![Color {
        red: false,
        green: false,
        blue: true,
        duration: 3000,
    }];
    let mut cur_seq = &cycle_seq;

    loop {
        for state in cur_seq {
            let mut out = false;
            led_driver.set_leds(state.red, state.green, state.blue);
            for _ in 1..state.duration / 10 {
                if let Ok(new_mode) = rx.try_recv() {
                    out = true;
                    match new_mode {
                        Mode::Cycle => cur_seq = &cycle_seq,
                        Mode::Red => cur_seq = &red_seq,
                        Mode::Yellow => cur_seq = &yellow_seq,
                        Mode::Green => cur_seq = &green_seq,
                        Mode::Blue => cur_seq = &blue_seq,
                    }
                    break;
                }
                FreeRtos::delay_ms(10);
            }
            if out {
                break;
            }
        }
    }
}

fn process_message(data: &[u8], details: Details, tx: &mpsc::Sender<Mode>) {
    if details == Complete {
        info!("{:?}", data);
        let message_data: &[u8] = data;
        if let Ok(command) = str::from_utf8(message_data) {
            info!("Command: {}", command);
            if command.eq("cycle") {
                tx.send(Mode::Cycle).unwrap();
            } else if command.eq("red") {
                tx.send(Mode::Red).unwrap();
            } else if command.eq("yellow") {
                tx.send(Mode::Yellow).unwrap();
            } else if command.eq("green") {
                tx.send(Mode::Green).unwrap();
            } else if command.eq("blue") {
                tx.send(Mode::Blue).unwrap();
            }
        }
    }
}
