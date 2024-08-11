This is a simple Rust app for controlling RGB LED via MQTT

Create cfg.toml (see cfg.toml.example) to specify your credentials

Publish 'cycle', 'red', 'yellow', 'green', 'blue' at 'esp32/martin_light' topic
to change LED color

You can use following command to publish to the MQTT topic:

mqttui -b mqtt://your.mqtt.broken.lan -u user --password password publish esp32/martin_light red
