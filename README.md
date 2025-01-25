# davyjones

`davyjones` is a simple [webhook receiver for Prometheus Alertmanager](https://prometheus.io/docs/alerting/latest/configuration/#webhook_config) to send notifications to a [ntfy](https://ntfy.sh) instance for ~~calling the kraken~~ mobile push notifications.

In addition to translating alerts `davyjones` also comes with the ability to configure dead man switches, which can be used to send notifications when a "watchdog" alert hasn't been triggered for a certain amount of time.

> [!WARNING]
> `davyjones` is in a very early state and subject to changes. Please don't expect a finished software product. If you want to use it I welcome issues and pull requests, but I cannot guarantee this project is secure or works correctly.

## Features

- [x] Basic webhook support for processing Alertmanager webhook calls.
- [ ] Watchdog support for sending alerts if no Alertmanager webhook calls came in for a while.

For feature requests, please open a [GitHub issue](https://github.com/embik/davyjones/issues).

## Usage

`davyjones` can either be built from source or be used from a container image hosted on [GitHub Packages](https://github.com/embik/davyjones/pkgs/container/davyjones). The container image can be used with Docker, Podman or Kubernetes.

By default, `davyjones` binds to `localhost`. You might have to tweak that with the `--host` flag. `davyjones` does **not** support basic authentication right now, so it is recommended to deploy it behind a reverse proxy that secures access.

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

### Alertmanager

`davyjones` exposes an endpoint (`/v1/webhooks/alerts`) for Alertmanager. To configure Alertmanager, add a new [receiver](https://prometheus.io/docs/alerting/latest/configuration/#receiver) to your Alertmanager configuration file. `davyjones` requires a [webhook_config](https://prometheus.io/docs/alerting/latest/configuration/#webhook_config), like this:

```yaml
- name: davyjones
  webhook_configs:
  - send_resolved: true
    http_config:
      # only if you run davyjones behind a reverse proxy with basic auth
      basic_auth:
        username: <username>
        password: <secret>
    url: https://<your davyjones hostname>/v1/webhooks/alerts
```
