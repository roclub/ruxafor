# ruxafor
This crate provides an API for controlling the [luxafor light](https://luxafor.com/). At the current development status, this API can only control USB-connected luxafor lights. Further development (such as controlling bluetooth connected devices) is in progress. Feel free to participate. 

## Usage
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
### Control mute button
However, if you want to check whether the mute button of the luxafor light is pressed for a period of time, you can simply use the `ìs_button_pressed(timeout)` method as presented in the following code block. Note that the variable `timeout` is measured in milliseconds.

```rust
use ruxafor::{USBDiscovery, Color};
use hidapi::HidError;

fn main() -> Result<(), HidError> {
    let usb_discovery = USBDiscovery::new()?;
    let usb_device = usb_discovery.device()?;
    if usb_device.is_button_pressed(1000, 5000) {
        // do something
    }
    Ok(())
}
```
