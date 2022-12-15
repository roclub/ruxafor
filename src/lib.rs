use hidapi::{HidApi, HidDevice, HidError};
use std::time::{Duration, Instant};

// Luxafor API constants.
const DEVICE_VENDOR_ID: u16 = 0x04d8;
const DEVICE_PRODUCT_ID: u16 = 0xf372;

const HID_REPORT_ID: u8 = 0;
const LED_ALL: u8 = 255;

const BUTTON_PRESSED: [u8; 8] = [131, 1, 0, 0, 0, 0, 0, 0];
const BUTTON_RELEASED: [u8; 8] = [131, 0, 0, 0, 0, 0, 0, 0];

const MODE_STATIC: u8 = 1;
const MODE_STROBE: u8 = 3;
const MODE_CIRCLING: u8 = 4;

const CIRCULAR_LENGTH_SHORT: u8 = 1;
const CIRCULAR_LENGTH_LONG: u8 = 2;
const CIRCULAR_LENGTH_OVERLAPPING_SHORT: u8 = 3;
const CIRCULAR_LENGTH_OVERLAPPING_LONG: u8 = 4;

#[derive(Clone, Debug, Copy)]
pub enum Color {
    Red,
    Green,
    Yellow,
    Blue,
    White,
    Cyan,
    Magenta,
    Custom {
        red: u8,
        green: u8,
        blue: u8,
    },
}

/// Length of the circular section used in the circular mode.
#[derive(Clone, Debug)]
pub enum CircularLength {
    /// Two LEDS are used
    Short,
    /// Four LEDS are used
    Long,
    /// Two LEDS are used, but they do not wait till the next wave starts.
    ShortOverlapping,
    /// Four LEDS are used, but they do not wait till the next wave starts.
    LongOverlapping,
}

/// USBDiscovery is used to discover the Luxafor device using the HID descriptor.
pub struct USBDiscovery {
    hid_api: HidApi,
}

/// Implementation of the luxafor USB device.
pub struct USBDevice {
    hid_device: HidDevice,
    id: String,
    target_led: u8,
}

impl USBDiscovery {
    /// Returns a USBDiscovery object.
    pub fn new() -> Result<Self, HidError> {
        let hid_api = HidApi::new()?;
        return Ok(Self { hid_api });
    }
    
    /// Opens a HID device using it's vendor id and product id.
    pub fn device(&self) -> Result<USBDevice, HidError> {
        let open_hid = self.hid_api.open(DEVICE_VENDOR_ID, DEVICE_PRODUCT_ID)?;
        return Ok(USBDevice::new(open_hid));
    }
}

impl USBDevice {
    /// Creates a USBDevice object with USB dependent identifiers.
    fn new(hid_device: HidDevice) -> Self {
        let id = format!(
            "{}::{}::{}",
            hid_device
                .get_manufacturer_string()
                .unwrap_or(Some("<error>".to_string()))
                .unwrap_or("<unknown>".to_string()),
            hid_device
                .get_product_string()
                .unwrap_or(Some("<error>".to_string()))
                .unwrap_or("<unknown>".to_string()),
            hid_device
                .get_serial_number_string()
                .unwrap_or(Some("<error>".to_string()))
                .unwrap_or("<unknown>".to_string()),
        );
        Self {
            hid_device,
            id,
            target_led: LED_ALL,
        }
    }

    /// Returns the USB device identifier.
    pub fn id(&self) -> String {
        self.id.clone()
    }

    /// Resolves the specified color to a rgb value.
    fn color_to_bytes(&self, color: Color) -> (u8, u8, u8) {
        match color {
            Color::Red => (255, 0, 0),
            Color::Green => (0, 255, 0),
            Color::Yellow => (255, 255, 0),
            Color::Blue => (0, 0, 255),
            Color::White => (255, 255, 255),
            Color::Cyan => (0, 255, 255),
            Color::Magenta => (255, 0, 255),
            Color::Custom { red, green, blue } => (red, green, blue),
        }
    }

    /// Blocking read on the device and returns the length of the payload.
    pub fn read(&self, buffer: &mut[u8]) -> Result<usize, HidError> {
        let res = self.hid_device.read(&mut buffer[..])?;
        return Ok(res);
    }

    /// Same as read but blocking is termianted after the timeout.
    pub fn read_timeout(&self, buffer: &mut[u8], timeout: i32) -> Result<usize, HidError> {
        let res = self.hid_device.read_timeout(&mut buffer[..], timeout)?;
        return Ok(res);
    }

     /// Checks whether the mute button is pressed for a period of time.
     pub fn is_button_pressed(&self, timeout: i32, interval: u64) -> Result<bool, HidError> {
        let mut buffer = [0u8; 8];

        let mut res = self.read_timeout(&mut buffer[..], timeout)?;
        let timestamp = Instant::now();
        if &buffer[..res] == BUTTON_PRESSED {
            res = self.read(&mut buffer[..])?;
            if &buffer[..res] == BUTTON_RELEASED
                && timestamp.elapsed() > Duration::from_millis(interval)
            {
                return Ok(true);
            }
        }
        return Ok(false);
    }

     /// Checks whether the mute button is pressed for a period of time and set feedback_color as feedback after the timeout.
     pub fn is_button_pressed_feedback(&self, timeout: i32, interval: u64, feedback_color: Color) -> Result<bool, HidError> {
        let mut buffer = [0u8; 8];

        let mut res = self.read_timeout(&mut buffer[..], timeout)?;
        let timestamp = Instant::now();
        if &buffer[..res] == BUTTON_PRESSED {
            self.set_static_color(Color::White)?;
            loop {
                res = self.read_timeout(&mut buffer[..], 1)?;
                if timestamp.elapsed() > Duration::from_millis(interval) {
                    self.set_static_color(feedback_color)?;
                    if &buffer[..res] == BUTTON_RELEASED {
                        return Ok(true);
                    }
                } else {
                    if &buffer[..res] == BUTTON_RELEASED {
                        break;
                    }
                }
            }
        }
        return Ok(false);
    }

    /// Bytes are written to the usb device.
    fn write(&self, buffer: &[u8]) -> Result<(), HidError> {
        self.hid_device.write(buffer)?;
        Ok(())
    }

    /// Sets a static luxafor light.
    pub fn set_static_color(&self, color: Color) -> Result<(), HidError> {
        let (r, g, b) = self.color_to_bytes(color);
        self.write(&[HID_REPORT_ID, MODE_STATIC, self.target_led, r, g, b])?;
        Ok(())
    }

    /// Sets the the luxafor light to a circling color mode.
    pub fn set_circling_color(&self, color: Color, pattern: CircularLength, circling_rate: u8, iterations: u8) -> Result<(), HidError> {
        let pattern = match pattern {
            CircularLength::Short => CIRCULAR_LENGTH_SHORT,
            CircularLength::Long => CIRCULAR_LENGTH_LONG,
            CircularLength::ShortOverlapping => CIRCULAR_LENGTH_OVERLAPPING_SHORT,
            CircularLength::LongOverlapping => CIRCULAR_LENGTH_OVERLAPPING_LONG,
        };
        let (r, g, b) = self.color_to_bytes(color);
        self.write(&[HID_REPORT_ID, MODE_CIRCLING, pattern, r, g, b, 0x00, iterations, circling_rate])?;
        Ok(())
    }

    /// Sets the luxafor light to a strobing pattern.
    pub fn set_strobe_color(&self, color: Color, strobe_speed: u8, iterations: u8) -> Result<(), HidError> {
        let (r, g, b) = self.color_to_bytes(color);
        self.write(&[HID_REPORT_ID, MODE_STROBE, self.target_led, r, g, b, strobe_speed, 0x00, iterations])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    // Unit tessts have to run serially instead of parallel because of the hidapi library.

    #[test]
    #[serial]
    fn test_discovery() -> Result<(), HidError> {
        let usb_discovery = USBDiscovery::new()?;
        let usb_device = usb_discovery.device()?;
        println!("{}", usb_device.id());
        Ok(())
    }

    #[test]
    #[serial]
    fn test_is_button_pressed() -> Result<(), HidError> {
        let usb_discovery = USBDiscovery::new()?;
        let usb_device = usb_discovery.device()?;
        assert_eq!(usb_device.is_button_pressed(3000, 2000)?, false);
        Ok(())
    }

    #[test]
    #[serial]
    fn test_is_button_pressed_feedback() -> Result<(), HidError> {
        let usb_discovery = USBDiscovery::new()?;
        let usb_device = usb_discovery.device()?;
        assert_eq!(usb_device.is_button_pressed_feedback(3000, 2000, Color::Red)?, false);
        Ok(())
    }

    #[test]
    #[serial]
    fn test_set_static_color() -> Result<(), HidError> {
        let usb_discovery = USBDiscovery::new()?;
        let usb_device = usb_discovery.device()?;
        usb_device.set_static_color(Color::Red)?;
        Ok(())
    }

    #[test]
    #[serial]
    fn test_set_circling_color() -> Result<(), HidError> {
        let usb_discovery = USBDiscovery::new()?;
        let usb_device = usb_discovery.device()?;
        usb_device.set_circling_color(Color::Red, CircularLength::Short, 1, 1)?;
        Ok(())
    }

    #[test]
    #[serial]
    fn test_set_strobe_color() -> Result<(), HidError> {
        let usb_discovery = USBDiscovery::new()?;
        let usb_device = usb_discovery.device()?;
        usb_device.set_strobe_color(Color::Red, 1, 1)?;
        Ok(())
    }
}