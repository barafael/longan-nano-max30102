#![no_std]
#![no_main]

use panic_halt as _;

use gd32vf103xx_hal::{gpio::{Alternate, OpenDrain, gpiob::{PB6, PB7}}, pac, prelude::*};
use riscv_rt::entry;

use max3010x::*;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    // Configure clocks
    let mut rcu = dp
        .RCU
        .configure()
        .ext_hf_clock(8.mhz())
        .sysclk(108.mhz())
        .freeze();
    let mut afio = dp.AFIO.constrain(&mut rcu);

    let mut delay = gd32vf103xx_hal::delay::McycleDelay::new(&rcu.clocks);

    let gpiob = dp.GPIOB.split(&mut rcu);

    let pb6 = gpiob.pb6.into_alternate_open_drain();
    let pb7 = gpiob.pb7.into_alternate_open_drain();

    let pins: (PB6<Alternate<OpenDrain>>, PB7<Alternate<OpenDrain>>) = (pb6, pb7);
    let i2c = gd32vf103xx_hal::i2c::BlockingI2c::i2c0(dp.I2C0, pins, &mut afio, gd32vf103xx_hal::i2c::Mode::Standard { frequency: 100u32.khz().into() }, &mut rcu, 100, 100, 100, 100);

    let sensor = Max3010x::new_max30102(i2c);
    let mut sensor = sensor.into_heart_rate().unwrap();

    sensor.set_sample_averaging(SampleAveraging::Sa4).unwrap();
    sensor.set_pulse_amplitude(Led::All, 15).unwrap();
    sensor.enable_fifo_rollover().unwrap();

    let mut data = [0; 3];
    loop {
        let samples_read = sensor.read_fifo(&mut data).unwrap();
        delay.delay_ms(100u32);
    }
}
