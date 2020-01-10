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
use hal::time::Hertz;

use hal::entry;

use mcp2517fd;
use mcp2517fd::generic::SFRAddress;
use mcp2517fd::spi;

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

    let mut d6 = pins.d6.into_push_pull_output(&mut pins.port);
    d6.set_high().unwrap();

    let mut d11 = pins.d11.into_push_pull_output(&mut pins.port);
    d11.set_low().unwrap();
    let mut d12 = pins.d12.into_push_pull_output(&mut pins.port);
    d12.set_low().unwrap();

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

    let mut controller = mcp2517fd::spi::Controller::new(master, d6);

    // match setup_can(&mut controller) {
    //     Ok(_) => d11.set_high().unwrap(),
    //     Err(_) => d12.set_high().unwrap(),
    // };

    let mut delay = Delay::new(core.SYST, &mut clocks);

    let settings = mcp2517fd::settings::Settings {
        oscillator: mcp2517fd::settings::Oscillator {
            pll: mcp2517fd::settings::PLL::Off,
            divider: mcp2517fd::settings::SysClkDivider::DivByOne,
        },
        ioconfiguration: mcp2517fd::settings::IOConfiguration {
            enable_tx_standby_pin: false,
            txcan_open_drain: false,
            sof_on_clko: false,
            interrupt_pin_open_drain: false,
        },
        txqueue: mcp2517fd::settings::TxQueueConfiguration {
            message_priority: 0u8,
            retransmission_attempts: mcp2517fd::can::control::RetransmissionAttempts::ThreeRetries,
            fifo_size: 0,
            payload_size: mcp2517fd::can::control::PayloadSize::Bytes8,
        },
        fifoconfigs: &[],
    };

    let mut configure_ok = false;
    match controller.configure(settings, &mut delay) {
        Ok(_) => {
            configure_ok = true;
            d11.set_high().unwrap();
        }
        Err(_) => d12.set_high().unwrap(),
    }

    while !configure_ok {
        delay.delay_ms(1000u32);
        d12.set_low().unwrap();
        match controller.verify_spi_communications() {
            Ok(_) => {
                configure_ok = true;
                d11.set_high().unwrap();
            }
            Err(_) => d12.set_high().unwrap(),
        }
    }

    loop {}
}
