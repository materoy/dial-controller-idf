use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{InputPin, OutputPin};
use esp_idf_svc::hal::prelude::Peripherals;
use crate::button::ButtonPressType;
use crate::joystick::joystick_loop;
use crate::keyboard::Keyboard;
use crate::rotary_encoder::RotaryEncoder;

mod keyboard;
mod rotary_encoder;
mod button;
mod joystick;

fn setup() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
}

fn main_loop(mut rotary_encoder: RotaryEncoder, mut keyboard: Keyboard) -> !
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
    // joystick_loop(peripherals)?;
    
    let rotary_encoder = 
        RotaryEncoder::new(peripherals);
    let keyboard = Keyboard::new()?;
    main_loop(rotary_encoder, keyboard)

}
