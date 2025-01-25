# davyjones

`davyjones` is a simple [webhook receiver for Alertmanager](https://prometheus.io/docs/alerting/latest/configuration/#webhook_config) to send notifications to a [ntfy](https://ntfy.sh) instance for ~~calling the kraken~~ mobile push notifications.

In addition to translating alerts `davyjones` also comes with the ability to configure dead man switches, which can be used to send notifications when a "watchdog" alert hasn't been triggered for a certain amount of time.

## Installation

TODO

## Configuration

`davyjones` is configured via a configuration file in TOML. An example can be found [here](./examples/config.toml).
