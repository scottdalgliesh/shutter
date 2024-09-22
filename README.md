# Shutter

A simple web server to display the live status of a wireless ESP32-based hall effect sensor. It can be used to check if your window is shut(ter).

## Description
* Intended to display the current status of ESP32C3-based wireless hall-effect sensor(s) setup per the instructions provided here: [shutter-sensor](https://github.com/scottdalgliesh/shutter-sensor).
* Frontend and server built with [Leptos](https://github.com/leptos-rs/leptos) and [Axum](https://github.com/tokio-rs/axum), respectively.
* Uses websockets to keep the sensor status readout live.

## Setup
* Clone this repo, and install the [Rust](https://www.rust-lang.org/learn/get-started) toolchain.
* Follow the steps in [Network Settings](## Network Settings) to configure server URL and configure network to accept incoming data from sensor hardware.
* Install cargo leptos via: `cargo install cargo-leptos`
* Run local server via: `cargo leptos serve`

## Network Settings

To expose the development server to other devices on a private network (may be required to receive data from sensor hardware), the following options must be configured. These instructions are for a windows 11 device. **This should only be done on a private network.**
1. Change the server IP address from `localhost:3000` to `<device_ip_address>:3000`, as described below:
    * Check device IP address via windows settings [instructions](https://support.microsoft.com/en-us/windows/find-your-ip-address-in-windows-f21a9bbc-c582-55cd-35e0-73431160a1b9)
    * Create a .env file in the project root directory specifying the server IP address in the following format: `"LEPTOS_SITE_ADDR=XXX.XXX.XXX.XXX:3000"`
    * Configure firewall to allow external traffic (external to device, but still within the private network) to the specified port ([instructions](https://learn.microsoft.com/en-us/sql/reporting-services/report-server/configure-a-firewall-for-report-server-access?view=sql-server-ver16))
2. Run the server (see above)
3. Visit the new IP address from any device **within the private network**


## Licensing

[MIT](https://choosealicense.com/licenses/mit/)