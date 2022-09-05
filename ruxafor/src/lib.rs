use hidapi::{HidApi, HidDevice, HidError};

// Luxafor API constants
const DEVICE_VENDOR_ID: u16 = 0x04d8;
const DEVICE_PRODUCT_ID: u16 = 0xf372;

const HID_REPORT_ID: u8 = 0;
const LED_ALL: u8 = 255;
const MODE_STATIC: u8 = 1;

#[derive(Clone, Debug)]
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

pub struct USBDiscovery {
    hid_api: HidApi,
}

pub struct USBDevice {
    hid_device: HidDevice,
    id: String,
    target_led: u8,
}

impl USBDiscovery {
    /// Returns a USBDiscovery object
    pub fn new() -> Result<Self, HidError> {
        let hid_api = HidApi::new()?;
        return Ok(Self { hid_api });
    }
    
    /// Opens a HID device using it's vendor id and product id
    pub fn device(&self) -> Result<USBDevice, HidError> {
        let open_hid = self.hid_api.open(DEVICE_VENDOR_ID, DEVICE_PRODUCT_ID)?;
        return Ok(USBDevice::new(open_hid));
    }
}

impl USBDevice {
    /// Creates a USBDevice object with USB dependent identifiers
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

    /// Resolves the specified color to a rgb value
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

    /// Bytes are written to the usb device
    fn write(&self, buffer: &[u8]) -> Result<(), HidError> {
        self.hid_device.write(buffer)?;
        Ok(())
    }

    /// Sets a static luxafor light
    pub fn set_static_color(&self, color: Color) -> Result<(), HidError> {
        let (r, g, b) = self.color_to_bytes(color);
        self.write(&[HID_REPORT_ID, MODE_STATIC, self.target_led, r, g, b])?;
        Ok(())
    }
}