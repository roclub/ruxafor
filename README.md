### Control mute button
However, if you want to check whether the mute button of the luxafor light is pressed for a period of time, you can simply use the `ìs_button_pressed(timeout)` method as presented in the following code block. Note that the variable `timeout` is measured in milliseconds.

```rust
use ruxafor::USBDiscovery;
use hidapi::HidError;

fn main() -> Result<(), HidError> {
    let usb_discovery = USBDiscovery::new()?;
    let usb_device = usb_discovery.device()?;
    if usb_device.is_button_pressed(5000) {
        // do something
    }
    Ok(())
}
```