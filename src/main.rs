use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{InputPin, OutputPin};
use esp_idf_svc::hal::prelude::Peripherals;
use crate::button::ButtonPressType;
use crate::keyboard::Keyboard;
use crate::rotary_encoder::RotaryEncoder;

mod keyboard;
mod rotary_encoder;
mod button;


fn setup() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
}

fn main_loop<T, K, A>(mut rotary_encoder: RotaryEncoder<T, K, A>, mut keyboard: Keyboard) -> !
where
    T: InputPin + OutputPin,
    K: InputPin + OutputPin,
    A: InputPin + OutputPin,
{
    loop {
        // Waits until ble is connected
        if !keyboard.connected() {
            FreeRtos::delay_ms(100);
            continue;
        }
        if let Some(press_type) = rotary_encoder.button.is_pressed() {
            match press_type {
                ButtonPressType::Normal => {
                    keyboard.press_enter();
                }
                ButtonPressType::Long => {
                    keyboard.press_escape();
                }
            }
        }
        rotary_encoder.handle_rotary_action(&mut keyboard).unwrap();
        FreeRtos::delay_ms(1);
    }
}

fn main() -> anyhow::Result<()> {
    setup();

    let peripherals = Peripherals::take()?;
    let rotary_encoder = RotaryEncoder::new(peripherals.pins.gpio2, peripherals.pins.gpio3, peripherals.pins.gpio1);
    let keyboard = Keyboard::new()?;
    main_loop(rotary_encoder, keyboard)

    // let y_pin = PinDriver::input(peripherals.pins.gpio2).unwrap();
    // let x_pin = PinDriver::input(peripherals.pins.gpio3).unwrap();
    // let button_pin = PinDriver::input(peripherals.pins.gpio1).unwrap();
    // let adc1 = AdcDriver::new(peripherals.adc1)?;
    //
    // let mut adc_config = AdcChannelConfig::default();
    // adc_config.calibration = true;
    // adc_config.attenuation = esp_idf_svc::hal::adc::attenuation::DB_11;
    // let mut adc_y_pin =
    //     AdcChannelDriver::new(&adc1, peripherals.pins.gpio2, &adc_config)?;
    //
    // let mut adc_x_pin =
    //     AdcChannelDriver::new(&adc1, peripherals.pins.gpio3, &adc_config)?;
    //
    // loop {
    //     sleep(Duration::from_millis(10));
    //     let adc_x = adc1.read(&mut adc_x_pin)?;
    //     let adc_y = adc1.read(&mut adc_y_pin)?;
    //     log::info!("X: {}   Y: {}", adc_x, adc_y);
    // }
}
