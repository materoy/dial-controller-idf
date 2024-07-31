use esp_idf_svc::hal::gpio::{Gpio1, Gpio2, Gpio3, Input, Level, PinDriver};
use esp_idf_svc::hal::peripherals::Peripherals;

use crate::button::Button;
use crate::keyboard::Keyboard;

pub struct RotaryEncoder<'d> {
    clk_pin: PinDriver<'d, Gpio2, Input>,
    dt_pin: PinDriver<'d, Gpio3, Input>,
    pub button: Button<'d, Gpio1>,
    counter: i32,
    prev_clk_state: Level,
}

impl<'d> RotaryEncoder<'d> {
    pub(crate) fn new(peripherals: Peripherals) -> Self {
        let clk_pin = PinDriver::input(peripherals.pins.gpio2).unwrap();
        let dt_pin = PinDriver::input(peripherals.pins.gpio3).unwrap();
        let button = Button::new(peripherals.pins.gpio1);
        let clk_state = clk_pin.get_level();
        RotaryEncoder {
            clk_pin,
            dt_pin,
            button,
            counter: 0,
            prev_clk_state: clk_state,
        }
    }

    pub(crate) fn handle_rotary_action(&mut self, keyboard: &mut Keyboard) -> anyhow::Result<()> {
        if self.clk_pin.get_level() == Level::Low && self.prev_clk_state == Level::High {
            if self.dt_pin.get_level() == Level::High {
                // Increment
                keyboard.press_arrow_forward();
                self.counter += 1;
                log::info!("{}", self.counter);
            } else {
                // Decrement
                keyboard.press_arrow_back();
                self.counter -= 1;
                log::info!("Dec: {}", self.counter);
            }
        }
        self.prev_clk_state = self.clk_pin.get_level();
        Ok(())
    }
}
