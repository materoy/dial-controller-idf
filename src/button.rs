use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{Input, InputPin, OutputPin, PinDriver, Pull};


// Threshold is not really in milliseconds, but cycle count
const LONG_PRESS_THRESHOLD: u32 = 20;
pub enum ButtonPressType {
    Normal,
    Long,
}

pub struct Button<'d, T>
where
    T: InputPin + OutputPin,
{
    pin: PinDriver<'d, T, Input>,
}

impl<'d, T> Button<'d, T>
where
    T: InputPin + OutputPin,
{
    pub fn new(pin: T) -> Self {
        let mut button = Button {
            pin: PinDriver::input(pin).unwrap(),
        };
        button.pin.set_pull(Pull::Up).unwrap();
        button
    }

    pub(crate) fn is_pressed(&self) -> Option<ButtonPressType> {
        if self.pin.is_high() { return None; }
        let mut counter: u32 = 0;
        while self.pin.is_low() {
            FreeRtos::delay_ms(1);
            counter += 1;
        }
        return if counter < LONG_PRESS_THRESHOLD {
            log::info!("Normal press: {}", counter);
            Some(ButtonPressType::Normal)
        } else {
            log::info!("Long press: {}", counter);
            Some(ButtonPressType::Long)
        };
    }
}
