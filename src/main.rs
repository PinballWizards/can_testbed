#![no_std]
#![no_main]

extern crate cortex_m;
extern crate cortex_m_semihosting;
extern crate embedded_hal;
extern crate feather_m0 as hal;
extern crate panic_halt;

#[macro_use]
extern crate nb;

use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::pac::{CorePeripherals, Peripherals};
use hal::prelude::*;

use hal::entry;

use embedded_hal::blocking::spi::{Transfer, Write};

use mcp2517fd;
use mcp2517fd::generic::SFRAddress;
use mcp2517fd::spi;

fn setup_can<T, SS>(
    controller: &mut mcp2517fd::spi::Controller<T, SS>,
    delay: &mut Delay,
) -> Result<(), spi::Error>
where
    T: Transfer<u8> + Write<u8>,
    SS: embedded_hal::digital::v2::StatefulOutputPin,
    <SS as embedded_hal::digital::v2::OutputPin>::Error: core::fmt::Debug,
{
    controller.reset()?;

    // Let's do GPIO first

    // Masks all configuration bits for OSC register
    // Use system clock with no PLL
    let mut osc = 0;
    // Enable system clock (wake from sleep)
    osc &= !(1 << 2);

    // Set up 10x CLKO divider
    osc |= 0b11 << 5;

    controller.write_sfr(&SFRAddress::OSC, osc)?;

    delay.delay_ms(5000u32);

    // Mask waiting for OSCRDY
    let osc_ready_mask = 1 << 10;

    // Wait for oscillator to give status ready
    osc = controller.read_sfr(&SFRAddress::OSC)?;
    while (osc & osc_ready_mask) != osc_ready_mask {
        delay.delay_ms(1000u32);
        osc = controller.read_sfr(&SFRAddress::OSC)?;
    }

    let mut iocon = controller.read_sfr(&SFRAddress::IOCON)?;
    // Set TRIS registers for output
    iocon &= !(0b11);

    let lat_mask = 0b11 << 8;

    // Set LAT registers
    iocon |= lat_mask;

    controller.write_sfr(&SFRAddress::IOCON, iocon)?;

    // Jump to normal mode
    let internal_loopback = 0b010;
    let mut c1con = controller.read_sfr(&SFRAddress::C1CON)?;

    let set_internal_loopback = |mut c1con: u32| -> u32 {
        c1con |= internal_loopback << 24;
        c1con &= !((!internal_loopback) << 24);
        c1con
    };
    controller.write_sfr(&SFRAddress::C1CON, set_internal_loopback(c1con))?;
    c1con = controller.read_sfr(&SFRAddress::C1CON)?;
    while c1con & (internal_loopback << 21) != (internal_loopback << 21) {
        delay.delay_ms(1000u32);
        controller.write_sfr(&SFRAddress::C1CON, set_internal_loopback(c1con))?;
        c1con = controller.read_sfr(&SFRAddress::C1CON)?;
        controller.read_sfr(&SFRAddress::C1TREC)?;
        controller.read_sfr(&SFRAddress::C1BDIAG0)?;
        controller.read_sfr(&SFRAddress::C1BDIAG1)?;
    }

    loop {
        iocon = controller.read_sfr(&SFRAddress::IOCON)?;
        delay.delay_ms(1000u32);

        if iocon & lat_mask != lat_mask {
            controller.write_sfr(&SFRAddress::IOCON, iocon | lat_mask)?;
        }
    }

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
        200.khz(),
        peripherals.SERCOM4,
        &mut peripherals.PM,
        pins.sck,
        pins.mosi,
        pins.miso,
        &mut pins.port,
    );

    let mut d6 = pins.d6.into_push_pull_output(&mut pins.port);
    d6.set_high().unwrap();

    let mut d11 = pins.d11.into_push_pull_output(&mut pins.port);
    d11.set_low().unwrap();
    let mut d12 = pins.d12.into_push_pull_output(&mut pins.port);
    d12.set_low().unwrap();

    let mut controller = mcp2517fd::spi::Controller::new(master, d6);

    // match setup_can(&mut controller) {
    //     Ok(_) => d11.set_high().unwrap(),
    //     Err(_) => d12.set_high().unwrap(),
    // };

    let mut delay = Delay::new(core.SYST, &mut clocks);

    let _ = setup_can(&mut controller, &mut delay);

    loop {
        match controller.read_sfr(&SFRAddress::OSC) {
            Ok(_) => {
                d11.set_high().unwrap();
                d12.set_low().unwrap();
            }
            Err(_) => {
                d12.set_high().unwrap();
                d11.set_low().unwrap();
            }
        }

        delay.delay_ms(1000u32);
        d11.set_low().unwrap();
        d12.set_low().unwrap();
        delay.delay_ms(1000u32);

        match controller.read_sfr(&SFRAddress::IOCON) {
            Ok(_) => {
                d11.set_high().unwrap();
                d12.set_low().unwrap();
            }
            Err(_) => {
                d12.set_high().unwrap();
                d11.set_low().unwrap();
            }
        }

        delay.delay_ms(1000u32);
        d11.set_low().unwrap();
        d12.set_low().unwrap();
        delay.delay_ms(1000u32);
    }

    // loop {
    //     match controller.read_sfr(&SFRAddress::C1CON) {
    //         Ok(_) => {
    //             d11.set_high().unwrap();
    //             d12.set_low().unwrap();
    //         }
    //         Err(_) => {
    //             d12.set_high().unwrap();
    //             d11.set_low().unwrap();
    //         }
    //     }

    //     delay.delay_ms(1000u32);
    //     d11.set_low().unwrap();
    //     d12.set_low().unwrap();
    //     delay.delay_ms(1000u32);
    // }
}
