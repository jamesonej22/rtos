//! Morse code timing and transmission for an Arduino UNO R4 Wifi LED.

#![no_std]

use core::{
    iter::Iterator,
    option::Option::{self, None, Some},
};
use embedded_hal::{delay::DelayNs, digital::OutputPin};

const TIME_UNIT_MILLIS: u32 = 132;
const DIT_DURATION_MILLIS: u32 = TIME_UNIT_MILLIS;
const DAH_DURATION_MILLIS: u32 = 3 * TIME_UNIT_MILLIS;
const INTRA_CHARACTER_GAP_MILLIS: u32 = TIME_UNIT_MILLIS;
const INTER_CHARACTER_GAP_MILLIS: u32 = 3 * TIME_UNIT_MILLIS;
pub const INTER_WORD_GAP_MILLIS: u32 = 7 * TIME_UNIT_MILLIS;

const MORSE: [&str; 36] = [
    ".-",    // a
    "-...",  // b
    "-.-.",  // c
    "-..",   // d
    ".",     // e
    "..-.",  // f
    "--.",   // g
    "....",  // h
    "..",    // i
    ".---",  // j
    "-.-",   // k
    ".-..",  // l
    "--",    // m
    "-.",    // n
    "---",   // o
    ".--.",  // p
    "--.-",  // q
    ".-.",   // r
    "...",   // s
    "-",     // t
    "..-",   // u
    "...-",  // v
    ".--",   // w
    "-..-",  // x
    "-.--",  // y
    "--..",  // z
    "-----", // 0
    ".----", // 1
    "..---", // 2
    "...--", // 3
    "....-", // 4
    ".....", // 5
    "-....", // 6
    "--...", // 7
    "---..", // 8
    "----.", // 9
];

fn char_to_morse(c: char) -> Option<&'static str> {
    let c = c.to_ascii_lowercase();
    if c.is_ascii_lowercase() {
        Some(MORSE[(c as usize) - ('a' as usize)])
    } else if c.is_ascii_digit() {
        Some(MORSE[26 + (c as usize) - ('0' as usize)])
    } else {
        None
    }
}

/// Displays one Morse pattern on the LED.
fn transmit_morse<L, D>(led: &mut L, delay: &mut D, morse: &str)
where
    L: OutputPin,
    D: DelayNs,
{
    for (i, symbol) in morse.chars().enumerate() {
        match symbol {
            '.' => {
                led.set_high().ok();
                delay.delay_ms(DIT_DURATION_MILLIS);
                led.set_low().ok();
            }
            '-' => {
                led.set_high().ok();
                delay.delay_ms(DAH_DURATION_MILLIS);
                led.set_low().ok();
            }
            _ => {}
        }
        if i < morse.len() - 1 {
            delay.delay_ms(INTRA_CHARACTER_GAP_MILLIS);
        }
    }
}

/// Converts and displays one supported letter or digit.
fn transmit_character<L, D>(led: &mut L, delay: &mut D, c: char)
where
    L: OutputPin,
    D: DelayNs,
{
    if let Some(morse) = char_to_morse(c) {
        transmit_morse(led, delay, morse);
    }
}

/// Displays a string in Morse code, using spaces as word separators.
pub fn transmit_string<L, D>(led: &mut L, delay: &mut D, message: &str)
where
    L: OutputPin,
    D: DelayNs,
{
    let mut chars = message.chars().peekable();

    while let Some(c) = chars.next() {
        if c == ' ' {
            delay.delay_ms(INTER_WORD_GAP_MILLIS);
            continue;
        }

        transmit_character(led, delay, c);
        if let Some(next) = chars.peek()
            && *next != ' '
        {
            delay.delay_ms(INTER_CHARACTER_GAP_MILLIS);
        }
    }
}
