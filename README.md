# davyjones

`davyjones` is a simple [webhook receiver for Alertmanager](https://prometheus.io/docs/alerting/latest/configuration/#webhook_config) to send notifications to a [ntfy](https://ntfy.sh) instance for ~~calling the kraken~~ mobile push notifications.

In addition to translating alerts `davyjones` also comes with the ability to configure dead man switches, which can be used to send notifications when a "watchdog" alert hasn't been triggered for a certain amount of time.

## Usage

`davyjones` can either be built from source or be used from a container image hosted on [GitHub Packages](https://github.com/embik/davyjones/pkgs/container/davyjones). The container image can be used with Docker, Podman or Kubernetes.

By default, `davyjones` binds to `localhost`. You might have to tweak that with the `--host` flag.

```sh
Usage: davyjones [OPTIONS]

Options:
  -v, --verbose          Enable verbose (debug) logging
  -c, --config <config>  Configuration file to start davyjones with
  -p, --port <port>      Port to listen on when starting webhook server [default: 8080]
      --host <host>      Host to listen on when starting webhook server [default: localhost]
  -h, --help             Print help
```

## Configuration

`davyjones` is configured via a configuration file in TOML. An example can be found [here](./examples/config.toml).
