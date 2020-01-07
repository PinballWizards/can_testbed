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
) -> Result<(), spi::Error<<T as Transfer<u8>>::Error, <T as Write<u8>>::Error, u8>>
where
    T: Transfer<u8> + Write<u8>,
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
    iocon &= !(1 << 0);
    iocon &= !(1 << 1);

    // LAT0/1 set as latched
    iocon |= 1 << 8;
    iocon |= 1 << 9;

    iocon |= 1 << 16;
    iocon |= 1 << 17;

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

    let mut master = hal::spi_master(
        &mut clocks,
        1.mhz(),
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

    // Keeping this delay here so slave select can go high.
    delay.delay_ms(100u32);

    // loop {
    //     d6.set_low().unwrap();
    //     match master.write(&[0b0101_0101 as u8; 1]) {
    //         Ok(_) => {
    //             d11.set_high().unwrap();
    //             d12.set_low().unwrap();
    //         }
    //         Err(_) => {
    //             d12.set_high().unwrap();
    //             d11.set_low().unwrap();
    //         }
    //     };
    //     d6.set_high().unwrap();
    //     delay.delay_ms(500u32);
    //     d11.set_low().unwrap();
    //     d12.set_low().unwrap();
    //     delay.delay_ms(500u32);
    // }

    // loop {
    //     match controller.write_sfr(
    //         &SFRAddress::IOCON,
    //         0b0101_0101_0101_0101_0101_0101_0101_0101 as u32,
    //     ) {
    //         Ok(_) => {
    //             d11.set_high().unwrap();
    //             d12.set_low().unwrap();
    //         }
    //         Err(_) => {
    //             d12.set_high().unwrap();
    //             d11.set_low().unwrap();
    //         }
    //     }

    //     delay.delay_ms(500u32);
    //     d11.set_low().unwrap();
    //     d12.set_low().unwrap();
    //     delay.delay_ms(500u32);
    // }

    let data = [
        (0b00000011_00000011_00000000_00000000 as u32),
        (0b00000011_00000011_00000001_00000000 as u32),
        (0b00000011_00000011_00000010_00000000 as u32),
        (0b00000011_00000011_00000011_00000000 as u32),
    ];

    // Let's do GPIO first
    let mut osc = match controller.read_sfr(&SFRAddress::OSC) {
        Ok(val) => val,
        Err(_) => 0,
    };

    delay.delay_ms(1000u32);
    // Masks all configuration bits for OSC register
    osc |= 0b000_0000;
    let _ = controller.write_sfr(&SFRAddress::OSC, osc);

    delay.delay_ms(1000u32);
    // Wait for oscillator to give status ready
    while (osc & (1 << 10)) == 0 {
        osc = match controller.read_sfr(&SFRAddress::OSC) {
            Ok(val) => val,
            Err(_) => 0,
        };
        delay.delay_ms(1000u32);
    }

    delay.delay_ms(1000u32);

    loop {
        // match controller.write_sfr(&SFRAddress::IOCON, data[3]) {
        //     Ok(_) => {
        //         d11.set_high().unwrap();
        //         d12.set_low().unwrap();
        //     }
        //     Err(_) => {
        //         d12.set_high().unwrap();
        //         d11.set_low().unwrap();
        //     }
        // };

        // delay.delay_ms(1000u32);
        // d11.set_low().unwrap();
        // d12.set_low().unwrap();
        // delay.delay_ms(1000u32);

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
