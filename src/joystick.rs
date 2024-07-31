use std::thread::sleep;
use std::time::Duration;

use esp_idf_svc::hal::adc::oneshot::{AdcChannelDriver, AdcDriver};
use esp_idf_svc::hal::adc::oneshot::config::AdcChannelConfig;
use esp_idf_svc::hal::peripherals::Peripherals;

pub fn joystick_loop(
    peripherals: Peripherals,
) -> anyhow::Result<()> where
{
    let adc1 = AdcDriver::new(peripherals.adc1)?;

    let mut adc_config = AdcChannelConfig::default();
    adc_config.calibration = true;
    adc_config.attenuation = esp_idf_svc::hal::adc::attenuation::DB_11;
    let mut adc_y_pin =
        AdcChannelDriver::new(&adc1, peripherals.pins.gpio2, &adc_config)?;

    let mut adc_x_pin =
        AdcChannelDriver::new(&adc1, peripherals.pins.gpio3, &adc_config)?;

    loop {
        sleep(Duration::from_millis(10));
        let adc_x = adc1.read(&mut adc_x_pin)?;
        let adc_y = adc1.read(&mut adc_y_pin)?;
        log::info!("X: {}   Y: {}", adc_x, adc_y);
    }
}
