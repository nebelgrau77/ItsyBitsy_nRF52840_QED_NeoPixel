#![no_main]
#![no_std]

use panic_halt as _;

use nrf52840_hal as hal;

use hal::{pac::{CorePeripherals, Peripherals},
        prelude::*,
        gpio::Level,
        delay::Delay,
        spim::Spim,
        qdec::*,
        };

use cortex_m_rt::entry;

use apa102_spi::Apa102;

use smart_leds_trait::RGB8;
use smart_leds::{
    hsv::{hsv2rgb, Hsv},
    SmartLedsWrite,
    };


#[entry]
fn main() -> ! {
    
    let p = Peripherals::take().unwrap();
    let core = CorePeripherals::take().unwrap();

    // set up GPIO ports
    let port0 = hal::gpio::p0::Parts::new(p.P0);
    let port1 = hal::gpio::p1::Parts::new(p.P1);
    
    // set up QED
    let pin_a = port0.p0_30.into_pullup_input().degrade(); // A1 on ItsyBitsy
    let pin_b = port0.p0_28.into_pullup_input().degrade(); // A2 on ItsyBitsy

    let mut qed = Qdec::new(p.QDEC, pin_a, pin_b, None, SamplePeriod::_128us);
    qed.debounce(true).enable_interrupt(NumSamples::_10smpl).enable();

    // set up pins for the NeoPixel 
    let di = port0.p0_08.into_push_pull_output(Level::Low).degrade(); // data line
    let ci = port1.p1_09.into_push_pull_output(Level::Low).degrade(); // clock
    let nc = port1.p1_14.into_floating_input().degrade(); // not connected

    // use these pins for the SPI bus
    let pins = hal::spim::Pins{
                sck: ci, 
                miso: Some(nc), 
                mosi: Some(di)};

    // set up SPI                
    let spi = Spim::new(
        p.SPIM2,
        pins,
        hal::spim::Frequency::K500,
        hal::spim::MODE_0,
        0,
    );

    // initialize a delay provider
    let mut delay = Delay::new(core.SYST);

    // set up the NeoPixel
    let mut dotstar = Apa102::new(spi);
    

    // HERE I MUST READ THE ENCODER VALUE, LIKE THIS:
    // let mut val: i16 = 0; 
    // val += qed.read();
    // AND INCREASE SOMETHING ACCORDINGLY
    // E.G. HUE: 
    // let color: [RGB8; 1] = [hsv2rgb(Hsv {hue: h, sat: 255,val: 16})];
    // dotstar.write(color.iter().cloned()).unwrap();

    let mut qed_val: i16 = 0;
    let mut color: [RGB8; 1] = [hsv2rgb(Hsv {hue: 0, sat: 255, val: 16})];
    
    // PROBLEM: QED readings must be signed integers, but the hue value must be u8. It must wrap to 0 or 255.

    loop {
        
        qed_val += qed.read();
        color = [hsv2rgb(Hsv {hue: qed_val as u8, sat: 255, val: 16})];
        dotstar.write(color.iter().cloned()).unwrap();
        delay.delay_ms(50 as u16);
        
    }
    
}