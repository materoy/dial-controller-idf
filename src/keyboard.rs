// originally: https://github.com/T-vK/ESP32-BLE-Keyboard
#![allow(dead_code)]

use std::sync::Arc;

use esp32_nimble::{
    BLEAdvertisementData, BLECharacteristic, BLEDevice, BLEHIDDevice, BLEServer, enums::*,
    hid::*, utilities::mutex::Mutex,
};

const KEYBOARD_ID: u8 = 0x01;
const MEDIA_KEYS_ID: u8 = 0x02;

const HID_REPORT_DISCRIPTOR: &[u8] = hid!(
  (USAGE_PAGE, 0x01), // USAGE_PAGE (Generic Desktop Ctrls)
  (USAGE, 0x06),      // USAGE (Keyboard)
  (COLLECTION, 0x01), // COLLECTION (Application)
  // ------------------------------------------------- Keyboard
  (REPORT_ID, KEYBOARD_ID), //   REPORT_ID (1)
  (USAGE_PAGE, 0x07),       //   USAGE_PAGE (Kbrd/Keypad)
  (USAGE_MINIMUM, 0xE0),    //   USAGE_MINIMUM (0xE0)
  (USAGE_MAXIMUM, 0xE7),    //   USAGE_MAXIMUM (0xE7)
  (LOGICAL_MINIMUM, 0x00),  //   LOGICAL_MINIMUM (0)
  (LOGICAL_MAXIMUM, 0x01),  //   Logical Maximum (1)
  (REPORT_SIZE, 0x01),      //   REPORT_SIZE (1)
  (REPORT_COUNT, 0x08),     //   REPORT_COUNT (8)
  (HIDINPUT, 0x02), //   INPUT (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
  (REPORT_COUNT, 0x01), //   REPORT_COUNT (1) ; 1 byte (Reserved)
  (REPORT_SIZE, 0x08), //   REPORT_SIZE (8)
  (HIDINPUT, 0x01), //   INPUT (Const,Array,Abs,No Wrap,Linear,Preferred State,No Null Position)
  (REPORT_COUNT, 0x05), //   REPORT_COUNT (5) ; 5 bits (Num lock, Caps lock, Scroll lock, Compose, Kana)
  (REPORT_SIZE, 0x01),  //   REPORT_SIZE (1)
  (USAGE_PAGE, 0x08),   //   USAGE_PAGE (LEDs)
  (USAGE_MINIMUM, 0x01), //   USAGE_MINIMUM (0x01) ; Num Lock
  (USAGE_MAXIMUM, 0x05), //   USAGE_MAXIMUM (0x05) ; Kana
  (HIDOUTPUT, 0x02), //   OUTPUT (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
  (REPORT_COUNT, 0x01), //   REPORT_COUNT (1) ; 3 bits (Padding)
  (REPORT_SIZE, 0x03), //   REPORT_SIZE (3)
  (HIDOUTPUT, 0x01), //   OUTPUT (Const,Array,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
  (REPORT_COUNT, 0x06), //   REPORT_COUNT (6) ; 6 bytes (Keys)
  (REPORT_SIZE, 0x08), //   REPORT_SIZE(8)
  (LOGICAL_MINIMUM, 0x00), //   LOGICAL_MINIMUM(0)
  (LOGICAL_MAXIMUM, 0x65), //   LOGICAL_MAXIMUM(0x65) ; 101 keys
  (USAGE_PAGE, 0x07), //   USAGE_PAGE (Kbrd/Keypad)
  (USAGE_MINIMUM, 0x00), //   USAGE_MINIMUM (0)
  (USAGE_MAXIMUM, 0x65), //   USAGE_MAXIMUM (0x65)
  (HIDINPUT, 0x00),  //   INPUT (Data,Array,Abs,No Wrap,Linear,Preferred State,No Null Position)
  (END_COLLECTION),  // END_COLLECTION
  // ------------------------------------------------- Media Keys
  (USAGE_PAGE, 0x0C),         // USAGE_PAGE (Consumer)
  (USAGE, 0x01),              // USAGE (Consumer Control)
  (COLLECTION, 0x01),         // COLLECTION (Application)
  (REPORT_ID, MEDIA_KEYS_ID), //   REPORT_ID (3)
  (USAGE_PAGE, 0x0C),         //   USAGE_PAGE (Consumer)
  (LOGICAL_MINIMUM, 0x00),    //   LOGICAL_MINIMUM (0)
  (LOGICAL_MAXIMUM, 0x01),    //   LOGICAL_MAXIMUM (1)
  (REPORT_SIZE, 0x01),        //   REPORT_SIZE (1)
  (REPORT_COUNT, 0x10),       //   REPORT_COUNT (16)
  (USAGE, 0xB5),              //   USAGE (Scan Next Track)     ; bit 0: 1
  (USAGE, 0xB6),              //   USAGE (Scan Previous Track) ; bit 1: 2
  (USAGE, 0xB7),              //   USAGE (Stop)                ; bit 2: 4
  (USAGE, 0xCD),              //   USAGE (Play/Pause)          ; bit 3: 8
  (USAGE, 0xE2),              //   USAGE (Mute)                ; bit 4: 16
  (USAGE, 0xE9),              //   USAGE (Volume Increment)    ; bit 5: 32
  (USAGE, 0xEA),              //   USAGE (Volume Decrement)    ; bit 6: 64
  (USAGE, 0x23, 0x02),        //   Usage (WWW Home)            ; bit 7: 128
  (USAGE, 0x94, 0x01),        //   Usage (My Computer) ; bit 0: 1
  (USAGE, 0x92, 0x01),        //   Usage (Calculator)  ; bit 1: 2
  (USAGE, 0x2A, 0x02),        //   Usage (WWW fav)     ; bit 2: 4
  (USAGE, 0x21, 0x02),        //   Usage (WWW search)  ; bit 3: 8
  (USAGE, 0x26, 0x02),        //   Usage (WWW stop)    ; bit 4: 16
  (USAGE, 0x24, 0x02),        //   Usage (WWW back)    ; bit 5: 32
  (USAGE, 0x83, 0x01),        //   Usage (Media sel)   ; bit 6: 64
  (USAGE, 0x8A, 0x01),        //   Usage (Mail)        ; bit 7: 128
  (HIDINPUT, 0x02), // INPUT (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
  (END_COLLECTION), // END_COLLECTION
);

pub const SHIFT: u8 = 0x80;

#[derive(Clone)]
enum RotaryInputKeys {
    FORWARD,
    BACK,
    SELECT,
}

impl RotaryInputKeys {
    fn as_byte(&self) -> u8 {
        match self {
            RotaryInputKeys::FORWARD => 0x01,
            RotaryInputKeys::BACK => 0x02,
            RotaryInputKeys::SELECT => 0x04,
        }
    }
}

#[repr(packed)]
struct KeyReport {
    modifiers: u8,
    reserved: u8,
    keys: [u8; 6],
}

pub struct Keyboard {
    server: &'static mut BLEServer,
    input_keyboard: Arc<Mutex<BLECharacteristic>>,
    output_keyboard: Arc<Mutex<BLECharacteristic>>,
    input_media_keys: Arc<Mutex<BLECharacteristic>>,
    key_report: KeyReport,
}

impl Keyboard {
    pub fn new() -> anyhow::Result<Self> {
        let device = BLEDevice::take();
        device
            .security()
            .set_auth(AuthReq::all())
            .set_io_cap(SecurityIOCap::NoInputNoOutput)
            .resolve_rpa();

        let server = device.get_server();
        let mut hid = BLEHIDDevice::new(server);

        let input_keyboard = hid.input_report(KEYBOARD_ID);
        let output_keyboard = hid.output_report(KEYBOARD_ID);
        let input_media_keys = hid.input_report(MEDIA_KEYS_ID);

        hid.manufacturer("Espressif");
        hid.pnp(0x02, 0x05ac, 0x820a, 0x0210);
        hid.hid_info(0x00, 0x01);

        hid.report_map(HID_REPORT_DISCRIPTOR);

        hid.set_battery_level(100);

        let ble_advertising = device.get_advertising();
        ble_advertising.lock().scan_response(true).set_data(
            BLEAdvertisementData::new()
                .name("Dial controller")
                .appearance(0x03C1)
                .add_service_uuid(hid.hid_service().lock().uuid()),
        )?;
        ble_advertising.lock().start()?;

        Ok(Self {
            server,
            input_keyboard,
            output_keyboard,
            input_media_keys,
            key_report: KeyReport {
                modifiers: 0,
                reserved: 0,
                keys: [0; 6],
            },
        })
    }

    pub fn connected(&self) -> bool {
        self.server.connected_count() > 0
    }

    pub fn send_char(&mut self, char: u8) {
        self.press_scan_code(char);
        self.release();
    }

    fn press_scan_code(&mut self, char: u8) {
        self.key_report.keys[0] = char;
        self.send_report(&self.key_report);
    }
    fn release(&mut self) {
        self.key_report.modifiers = 0;
        self.key_report.keys.fill(0);
        self.send_report(&self.key_report);
    }

    fn send_report(&self, keys: &KeyReport) {
        self.input_keyboard.lock().set_from(keys).notify();
        esp_idf_svc::hal::delay::Ets::delay_ms(7);
    }
    
    pub fn press_enter(&mut self) {
        self.send_char(0x28); // Enter
    }
    
    pub fn press_arrow_back(&mut self) {
        self.send_char(0x50); // Arrow Left
        self.send_char(0xe1 | 0x2b); // Shift TAB
    }
    
    pub fn press_arrow_forward(&mut self) {
        // self.send_char(0x4f); // Arrow Right
        self.send_char(0x2b); // TAB
    }
    
    pub fn press_escape(&mut self) {
        self.send_char(0x29); // Backspace
    }
}
