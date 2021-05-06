#![no_std]
#![no_main]

use panic_halt as _;

use gd32vf103xx_hal::i2c::DutyCycle;
use gd32vf103xx_hal::{
    gpio::{
        gpiob::{PB6, PB7},
        Alternate, OpenDrain,
    },
    pac,
    prelude::*,
};
use riscv_rt::entry;

use max3010x::*;

use core::fmt::Write;
use embedded_graphics::fonts::{Font8x16, Text};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::{primitive_style, text_style};
use heapless::{consts::U100 as heapless_100, String};
use longan_nano::{lcd, lcd_pins};

const W: usize = 160;

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

    let gpioa = dp.GPIOA.split(&mut rcu);
    let gpiob = dp.GPIOB.split(&mut rcu);

    let lcd_pins = lcd_pins!(gpioa, gpiob);
    let mut lcd = lcd::configure(dp.SPI0, lcd_pins, &mut afio, &mut rcu);
    let (_, height) = (lcd.size().width as i32, lcd.size().height as i32);

    let style = text_style!(
        font = Font8x16,
        text_color = Rgb565::BLACK,
        background_color = Rgb565::GREEN
    );

    Rectangle::new(Point::new(0, 0), Point::new(W as i32 - 1, height - 1))
        .into_styled(primitive_style!(fill_color = Rgb565::BLACK))
        .draw(&mut lcd)
        .unwrap();

    let pb6 = gpiob.pb6.into_alternate_open_drain();
    let pb7 = gpiob.pb7.into_alternate_open_drain();

    let pins: (PB6<Alternate<OpenDrain>>, PB7<Alternate<OpenDrain>>) = (pb6, pb7);
    let i2c = gd32vf103xx_hal::i2c::BlockingI2c::i2c0(
        dp.I2C0,
        pins,
        &mut afio,
        gd32vf103xx_hal::i2c::Mode::Fast {
            frequency: 400.khz().into(),
            duty_cycle: DutyCycle::Ratio2to1,
        },
        &mut rcu,
        1000,
        10,
        1000,
        1000,
    );

    let mut sensor = Max3010x::new_max30102(i2c);
    sensor.reset().unwrap();
    delay.delay_ms(100u8);

    let mut sensor = sensor.into_heart_rate().unwrap();

    sensor.set_sample_averaging(SampleAveraging::Sa8).unwrap();
    sensor.set_pulse_amplitude(Led::All, 15).unwrap();
    sensor.enable_fifo_rollover().unwrap();
    sensor.set_sampling_rate(SamplingRate::Sps100).unwrap();
    sensor.set_pulse_width(LedPulseWidth::Pw411).unwrap();

    sensor.clear_fifo().unwrap();

    let mut buf = String::<heapless_100>::new();
    let id = sensor.get_part_id().unwrap_or_default();
    let rev = sensor.get_revision_id().unwrap_or_default();
    let _ = write!(buf, "ID: {}, rev: {}", id, rev);

    Text::new(&buf, Point::new(2, 2))
        .into_styled(style)
        .draw(&mut lcd)
        .unwrap();

    loop {
        delay.delay_ms(100u8);
        let mut buf = String::<heapless_100>::new();
        let mut data = [0; 16];
        let samples_read = sensor.read_fifo(&mut data).unwrap_or(0);

        let _avg: u32 = ((data.iter().take(samples_read.into()).sum::<u32>() * 1000u32)
            / samples_read as u32)
            / 1000u32;
        let _ = write!(buf, "value: {}: {}", samples_read, data[0]);

        Text::new("        ", Point::new(2, 32))
            .into_styled(style)
            .draw(&mut lcd)
            .unwrap();
        Text::new(&buf, Point::new(2, 32))
            .into_styled(style)
            .draw(&mut lcd)
            .unwrap();
    }
}
