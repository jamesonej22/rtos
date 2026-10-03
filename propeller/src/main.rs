#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use core::cell::Cell;

use arduino_hal::{Peripherals, default_serial, pins};
use avr_device::interrupt::Mutex;
use panic_halt as _;
use ufmt::uwriteln;

static BEAM_BLOCKED: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));
static STATE_CHANGED: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));
static BREAK_COUNT: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));
static LAST_EDGE_TICKS: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));
static LAST_BREAK_TICKS: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));
static TIMER1_OVERFLOWS: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));
static RPM: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));

/// Number of blade interruptions per revolution:
/// For a pencil use 1, for a 3-blade propeller, use 3.
const INTERRUPTIONS_PER_REVOLUTION: u32 = 1;
const DEBOUNCE_TICKS: u32 = 40_000;
const SENSOR_BIT: u8 = 2;
const DEBUG_BIT: u8 = 7;

// Timer1 overflow interrupt
#[avr_device::interrupt(atmega328p)]
fn TIMER1_OVF() {
    avr_device::interrupt::free(|cs| {
        let overflows = TIMER1_OVERFLOWS.borrow(cs).get();
        TIMER1_OVERFLOWS.borrow(cs).set(overflows.wrapping_add(1));
    });
}

// D2 / PD2 is configured for "any logical change"
#[avr_device::interrupt(atmega328p)]
fn INT0() {
    let dp = unsafe { Peripherals::steal() };

    // Set D7 HIGH immediately upon entering the ISR
    dp.PORTD
        .portd()
        .modify(|r, w| unsafe { w.bits(r.bits() | (1 << DEBUG_BIT)) });

    avr_device::interrupt::free(|cs| {
        let timer = dp.TC1.tcnt1().read().bits() as u32;
        let overflows = TIMER1_OVERFLOWS.borrow(cs).get();
        let now_ticks = (overflows << 16) | timer;
        let last_edge = LAST_EDGE_TICKS.borrow(cs).get();

        // Ignore changes that arrive too soon after the last accepted one.
        if now_ticks.wrapping_sub(last_edge) >= DEBOUNCE_TICKS {
            LAST_EDGE_TICKS.borrow(cs).set(now_ticks);

            // Read D2
            let pind = dp.PORTD.pind().read().bits();
            let is_blocked = (pind & (1 << SENSOR_BIT)) != 0;
            let was_blocked = BEAM_BLOCKED.borrow(cs).get();

            if is_blocked != was_blocked {
                BEAM_BLOCKED.borrow(cs).set(is_blocked);
                if is_blocked {
                    let count = BREAK_COUNT.borrow(cs).get();
                    BREAK_COUNT.borrow(cs).set(count.wrapping_add(1));

                    // Calculate time since previous blade passage
                    let previous = LAST_BREAK_TICKS.borrow(cs).get();
                    if previous != 0 {
                        let period_ticks = now_ticks.wrapping_sub(previous);
                        if period_ticks > 0 {
                            let rpm =
                                2_000_000u32 * 60 / (period_ticks * INTERRUPTIONS_PER_REVOLUTION);
                            RPM.borrow(cs).set(rpm);
                        }
                    }

                    LAST_BREAK_TICKS.borrow(cs).set(now_ticks);
                }
                STATE_CHANGED.borrow(cs).set(true);
            }
        }
    });

    // Return D7 LOW immediately before leaving the ISR
    dp.PORTD
        .portd()
        .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << 7)) });
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();

    // D2 / PD2 = IR detector input
    dp.PORTD
        .ddrd()
        .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << SENSOR_BIT)) });

    // D7 / PD7 = oscilloscope debug output
    dp.PORTD
        .ddrd()
        .modify(|r, w| unsafe { w.bits(r.bits() | (1 << DEBUG_BIT)) });
    // Start D7 LOW.
    dp.PORTD
        .portd()
        .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << DEBUG_BIT)) });

    // D2 = INT0, ISC01:ISC00 = 01 means "any logical change".
    dp.EXINT
        .eicra()
        .modify(|r, w| unsafe { w.bits((r.bits() & !0b11) | 0b01) });
    dp.EXINT
        .eimsk()
        .modify(|r, w| unsafe { w.bits(r.bits() | 0b01) });

    let pins = pins!(dp);
    let mut serial = default_serial!(dp, pins, 115200);
    // D9 used as visual indicator: HIGH = beam clear, LOW = beam blocked
    let mut led = pins.d9.into_output();

    // Timer1 configuration
    let tc1 = dp.TC1;
    tc1.tccr1a().write(|w| unsafe { w.wgm1().bits(0b00) });
    tc1.tccr1b()
        .write(|w| unsafe { w.wgm1().bits(0b00).cs1().prescale_8() });
    tc1.timsk1().write(|w| w.toie1().set_bit());

    // Enable global interrups
    unsafe {
        avr_device::interrupt::enable();
    }

    // Assume the beam is clear at startup
    led.set_high();
    uwriteln!(&mut serial, "t_ms,state,count,rpm\r").unwrap();

    loop {
        // Check whether the ISR reported a state transition.
        let changed = avr_device::interrupt::free(|cs| {
            let c = STATE_CHANGED.borrow(cs).get();
            if c {
                STATE_CHANGED.borrow(cs).set(false);
            }
            c
        });

        if changed {
            let (blocked, count, rpm, timestamp) = avr_device::interrupt::free(|cs| {
                // Get current extended Timer1 timestamp
                let timer = tc1.tcnt1().read().bits() as u32;
                let overflows = TIMER1_OVERFLOWS.borrow(cs).get();
                let now_ticks = (overflows << 16) | timer;
                (
                    BEAM_BLOCKED.borrow(cs).get(),
                    BREAK_COUNT.borrow(cs).get(),
                    RPM.borrow(cs).get(),
                    now_ticks,
                )
            });

            let timestamp_ms = timestamp / 2_000;
            if blocked {
                led.set_low();
                uwriteln!(&mut serial, "{},BLOCKED,{},{}\r", timestamp_ms, count, rpm).unwrap();
            } else {
                led.set_high();
                uwriteln!(&mut serial, "{},CLEAR,{},{}\r", timestamp_ms, count, rpm).unwrap();
            }
        }
    }
}
