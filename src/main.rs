use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{Input, InputPin, Level, OutputPin, PinDriver, Pull};
use esp_idf_svc::hal::prelude::Peripherals;

mod keyboard;


struct RotaryEncoder<'d, T, K, A>
where
    T: InputPin + OutputPin,
    K: InputPin + OutputPin,
    A: InputPin + OutputPin,
{
    clk_pin: PinDriver<'d, T, Input>,
    dt_pin: PinDriver<'d, K, Input>,
    button: Button<'d, A>,
    counter: i32,
    prev_clk_state: Level,
}

impl<'d, T, K, A> RotaryEncoder<'d, T, K, A>
where
    T: InputPin + OutputPin,
    K: InputPin + OutputPin,
    A: InputPin + OutputPin,
{
    fn new(clk_pin: T, dt_pin: K, sw_pin: A) -> Self {
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

    fn handle_rotary_action(&mut self) {
        if self.clk_pin.get_level() == Level::Low && self.prev_clk_state == Level::High {
            if self.dt_pin.get_level() == Level::Low  {
                // Increment
                self.counter += 1;
                log::info!("{}", self.counter);
            } else {
                // Decrement
                self.counter -= 1;
                log::info!("Dec: {}", self.counter);
            }
        }
        self.prev_clk_state = self.clk_pin.get_level();
    }
}


struct Button<'d, T>
where
    T: InputPin + OutputPin,
{
    pin: PinDriver<'d, T, Input>,
}

impl<'d, T> Button<'d, T>
where
    T: InputPin + OutputPin,
{
    fn new(pin: T) -> Self {
        let mut button = Button {
            pin: PinDriver::input(pin).unwrap(),
        };
        button.pin.set_pull(Pull::Up).unwrap();
        button
    }

    fn is_pressed(&self) -> bool {
        return if self.pin.is_low() {
            FreeRtos::delay_ms(10);
            true
        } else {
            false
        };
    }
}

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let mut rotary_encoder = RotaryEncoder::new(peripherals.pins.gpio2, peripherals.pins.gpio3, peripherals.pins.gpio1);

    // let mut keyboard = Keyboard::new()?;

    loop {
        // if !keyboard.connected() {
        //     FreeRtos::delay_ms(100);
        //     continue;
        // }
        if rotary_encoder.button.is_pressed() {
            log::info!("Button pressed");
        }
        rotary_encoder.handle_rotary_action();
        FreeRtos::delay_ms(1);
    }
}
