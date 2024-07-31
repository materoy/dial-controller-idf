use esp_idf_svc::hal::gpio::{Input, InputPin, Level, OutputPin, PinDriver};
use crate::button::Button;
use crate::keyboard::Keyboard;

pub struct RotaryEncoder<'d, T, K, A>
where
    T: InputPin + OutputPin,
    K: InputPin + OutputPin,
    A: InputPin + OutputPin,
{
    clk_pin: PinDriver<'d, T, Input>,
    dt_pin: PinDriver<'d, K, Input>,
    pub(crate) button: Button<'d, A>,
    counter: i32,
    prev_clk_state: Level,
}

impl<'d, T, K, A> RotaryEncoder<'d, T, K, A>
where
    T: InputPin + OutputPin,
    K: InputPin + OutputPin,
    A: InputPin + OutputPin,
{
    pub(crate) fn new(clk_pin: T, dt_pin: K, sw_pin: A) -> Self {
        let clk_pin = PinDriver::input(clk_pin).unwrap();
        let dt_pin = PinDriver::input(dt_pin).unwrap();
        let button = Button::new(sw_pin);
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
