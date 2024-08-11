use embedded_svc::mqtt::client::{
    Details::Complete, EventPayload::Error, EventPayload::Received, QoS,
};
use esp_idf_hal::delay::FreeRtos;
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

enum Mode {
    Cycle,
    Red,
    Yellow,
    Green,
    Blue,
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

    let mut cur_seq = get_cycle_seq();

    loop {
        for state in cur_seq.clone() {
            let mut out = false;
            led_driver.set_leds(state.red, state.green, state.blue);
            for _ in 1..state.duration / 10 {
                if let Ok(new_mode) = rx.try_recv() {
                    out = true;
                    match new_mode {
                        Mode::Cycle => cur_seq = get_cycle_seq(),
                        Mode::Red => cur_seq = get_red_seq(),
                        Mode::Yellow => cur_seq = get_yellow_seq(),
                        Mode::Green => cur_seq = get_green_seq(),
                        Mode::Blue => cur_seq = get_blue_seq(),
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
