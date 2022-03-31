#![no_std]
#![no_main]

use adafruit_alphanum4::{AlphaNum4, AsciiChar, Index};
use cortex_m_rt::entry;
use defmt::info;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use embedded_time::duration::*;
use embedded_time::rate::Extensions;
use ht16k33::{Dimming, Display, HT16K33};
use num_traits::float::FloatCore;
use rp_pico::hal;
use rp_pico::hal::pac;
use rp_pico::hal::prelude::*;
use {defmt_rtt as _, panic_probe as _};

#[entry]
fn main() -> ! {
	let mut pac = pac::Peripherals::take().unwrap();
	let core = pac::CorePeripherals::take().unwrap();

	let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

	let clocks = hal::clocks::init_clocks_and_plls(
		rp_pico::XOSC_CRYSTAL_FREQ,
		pac.XOSC,
		pac.CLOCKS,
		pac.PLL_SYS,
		pac.PLL_USB,
		&mut pac.RESETS,
		&mut watchdog,
	)
	.ok()
	.unwrap();

	let sio = hal::Sio::new(pac.SIO);

	let pins = rp_pico::Pins::new(
		pac.IO_BANK0,
		pac.PADS_BANK0,
		sio.gpio_bank0,
		&mut pac.RESETS,
	);

	let i2c = hal::I2C::i2c0(
		pac.I2C0,
		pins.gpio16.into_mode::<hal::gpio::FunctionI2C>(),
		pins.gpio17.into_mode::<hal::gpio::FunctionI2C>(),
		20.kHz(),
		&mut pac.RESETS,
		clocks.peripheral_clock,
	);

	let mut ht16k33 = HT16K33::new(i2c, 112u8);
	ht16k33.initialize().expect("Failed to initialize ht16k33");
	ht16k33
		.set_display(Display::ON)
		.expect("Could not turn on the display!");
	ht16k33
		.set_dimming(Dimming::BRIGHTNESS_MIN)
		.expect("Could not set dimming!");
	ht16k33.clear_display_buffer();

	let switch_pin = pins.gpio15.into_pull_up_input();

	// Sending individual digits
	let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().integer());

	info!("Setup everything");
	pins.led
		.into_push_pull_output()
		.set_high()
		.expect("Failed to set LED to high");

	let mut index: u32 = 900;
	loop {
		if switch_pin.is_low().unwrap() {
			ht16k33.clear_display_buffer();
			ht16k33.update_buffer_with_char(Index::One, AsciiChar::new('S'));
			ht16k33.update_buffer_with_char(Index::Two, AsciiChar::new('T'));
			ht16k33.update_buffer_with_char(Index::Three, AsciiChar::new('O'));
			ht16k33.update_buffer_with_char(Index::Four, AsciiChar::new('P'));
			info!("Is low");
		} else {
			info!("Is high, {}", index);
			index -= 1;
			ht16k33
				.update_buffer_with_float(Index::One, (index as f32 / 60.0).round(), 2, 10)
				.expect("Failed to update display");
			delay.delay_ms(995);
		}
		ht16k33.write_display_buffer().unwrap();
	}
}
