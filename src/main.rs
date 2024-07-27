use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{Input, InputPin, Level, OutputPin, PinDriver, Pull};
use esp_idf_svc::hal::prelude::Peripherals;

use crate::keyboard::Keyboard;

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

    fn handle_rotary_action(&mut self, keyboard: &mut Keyboard) -> anyhow::Result<()> {
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

// Threshold is not really in milliseconds, but cycle count
const LONG_PRESS_THRESHOLD: u32 = 20;
enum ButtonPressType {
    Normal,
    Long,
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

    fn is_pressed(&self) -> Option<ButtonPressType> {
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
}
