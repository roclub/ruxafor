# ruxafor
This crate provides an API for controlling the [luxafor light](https://luxafor.com/). At the current development status, this API can only control USB-connected luxafor lights. Further development (such as controlling bluetooth connected devices) is in progress. Feel free to participate. 

## Usage
First of all, specify the dependencies in your `Cargo.toml`. Since ruxafor hasn't been published to [crates.io](https://crates.io/) yet, you have to import the crate via git.
```toml
ruxafor = { git = "https://github.com/roclub/ruxafor" }
hidapi = "1.4.2"
```

For basic usage (e.g. switching the color of your luxafor light to red) take a look at the follwoing code block.
```rust
use ruxafor::{USBDiscovery, Color};
use hidapi::HidError;

fn main() -> Result<(), HidError> {
    let usb_discovery = USBDiscovery::new()?;
    let usb_device = usb_discovery.device()?;
    usb_device.set_static_color(Color::Red)?;
    Ok(()) 
}

```
