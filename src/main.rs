use embedded_svc::mqtt::client::{Details::Complete, EventPayload::*, QoS};
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::*;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::mqtt::client::{Details, EspMqttClient, MqttClientConfiguration};
use log::{info, warn};
use std::str;
use std::sync::mpsc;
use wifi::wifi;

mod led_driver;
use led_driver::LedDriver;

mod sequences;
use sequences::*;

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

#[derive(Clone)]
enum Message {
    Cycle,
    Red,
    Yellow,
    Green,
    Blue,
    Purple,
    Connected,
}

impl From<String> for Message {
    fn from(string: String) -> Self {
        if string.eq("cycle") {
            Message::Cycle
        } else if string.eq("red") {
            Message::Red
        } else if string.eq("yellow") {
            Message::Yellow
        } else if string.eq("green") {
            Message::Green
        } else if string.eq("blue") {
            Message::Blue
        } else if string.eq("purple") {
            Message::Purple
        } else {
            // Default to Cycle for unknown
            warn!("Unknown mode: {}, defaulting to Cycle", string);
            Message::Cycle
        }
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

    let mut white_led = PinDriver::output(peripherals.pins.gpio19).unwrap();
    white_led.set_low().unwrap();

    let (tx, rx) = mpsc::channel::<Message>();
    let mut client =
        EspMqttClient::new_cb(
            &broker_url,
            &mqtt_config,
            move |message_event| match message_event.payload() {
                Connected(_) => tx.send(Message::Connected).unwrap(),
                Received { data, details, .. } => process_message(data, details, &tx),
                Error(e) => warn!("Received error from MQTT: {:?}", e),
                _ => info!("Received from MQTT: {:?}", message_event.payload()),
            },
        )
        .unwrap();

    let mut cur_seq = get_cycle_seq();

    loop {
        for state in cur_seq.clone() {
            let mut out = false;
            led_driver.set_leds(state.red, state.green, state.blue);
            for _ in 1..state.duration / 10 {
                if let Ok(new_mode) = rx.try_recv() {
                    out = true;
                    cur_seq = match new_mode {
                        Message::Cycle => get_cycle_seq(),
                        Message::Red => get_red_seq(),
                        Message::Yellow => get_yellow_seq(),
                        Message::Green => get_green_seq(),
                        Message::Blue => get_blue_seq(),
                        Message::Purple => get_purple_seq(),
                        Message::Connected => {
                            out = false;
                            client
                                .subscribe("esp32/martin_light", QoS::AtLeastOnce)
                                .unwrap();
                            cur_seq
                        }
                    };
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

fn process_message(data: &[u8], details: Details, tx: &mpsc::Sender<Message>) {
    if details == Complete {
        let message_data: &[u8] = data;
        if let Ok(mode) = String::from_utf8(message_data.into()) {
            info!("mode: {}", mode);
            tx.send(mode.into()).unwrap();
        }
    }
}
