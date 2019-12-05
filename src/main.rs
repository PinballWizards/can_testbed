#![no_std]
#![no_main]

extern crate cortex_m;
extern crate cortex_m_semihosting;
extern crate embedded_hal;
extern crate feather_m0 as hal;
extern crate panic_halt;

use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::pac::{CorePeripherals, Peripherals};
use hal::prelude::*;

use hal::entry;

use mcp2517fd;
use mcp2517fd::generic::SFRAddress;

fn setup_can<T, SS>(controller: &mut mcp2517fd::spi::Controller<T, SS>) -> Result<(), T::Error>
where
    T: embedded_hal::spi::FullDuplex<u8>,
    SS: embedded_hal::digital::v2::StatefulOutputPin,
    <SS as embedded_hal::digital::v2::OutputPin>::Error: core::fmt::Debug,
{
    // Let's do GPIO first
    let mut osc = controller.read_sfr(&SFRAddress::OSC)?;
    // Masks all configuration bits for OSC register
    osc |= 0b000_0000;
    let _ = controller.write_sfr(&SFRAddress::OSC, osc)?;

    // Wait for oscillator to give status ready
    while (osc & (1 << 10)) == 0 {
        osc = controller.read_sfr(&SFRAddress::OSC)?;
    }

    let mut iocon = controller.read_sfr(&SFRAddress::IOCON)?;
    // TRIS0/1 set GPIO0/1 as output
    iocon |= 1 << 0;
    iocon |= 1 << 1;

    // LAT0/1 set as latched
    iocon |= 1 << 8;
    iocon |= 1 << 9;

    // PM0/1 set as GPIO
    iocon |= 1 << 24;
    iocon |= 1 << 25;

    // Ensure interrupt pins are in push/pull mode
    iocon &= !(1 << 30);

    controller.write_sfr(&SFRAddress::IOCON, iocon)?;

    Ok(())
}

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_external_32kosc(
        peripherals.GCLK,
        &mut peripherals.PM,
        &mut peripherals.SYSCTRL,
        &mut peripherals.NVMCTRL,
    );

    let mut pins = hal::Pins::new(peripherals.PORT);

    let master = hal::spi_master(
        &mut clocks,
        1.mhz(),
        peripherals.SERCOM4,
        &mut peripherals.PM,
        pins.sck,
        pins.mosi,
        pins.miso,
        &mut pins.port,
    );

    let d6 = pins.d6.into_push_pull_output(&mut pins.port);

    let mut d11 = pins.d11.into_push_pull_output(&mut pins.port);
    d11.set_low().unwrap();
    let mut d12 = pins.d12.into_push_pull_output(&mut pins.port);
    d12.set_low().unwrap();

    let mut controller = mcp2517fd::spi::Controller::new(master, d6);

    match setup_can(&mut controller) {
        Ok(_) => d11.set_high().unwrap(),
        Err(_) => d12.set_high().unwrap(),
    };

    loop {}
}
