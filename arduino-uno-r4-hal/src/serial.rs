//! UART (SCI2 = Arduino Serial1, pins D0/D1 = P301/P302).
//!
//! Provides asynchronous 8N1 send/receive. Implements embedded-io, embedded-hal-nb, and
//! core::fmt::Write, so `write!`/`writeln!` work directly.
//!
//! SCI9 (P109/P110) is used by the UNO R4 WiFi USB bridge.
//!
//! # Baud rate
//!
//! The RA4M1 SCI's asynchronous baud rate follows (with SEMR.ABCS=1, BGDM=1):
//!
//! ```text
//! BRR = PCLKB / (divisor * baud) - 1,  divisor = 16 * 2^(2n-1) = 8 << (2n)
//! ```
//!
//! `n` (= SMR.CKS, 0..3) is chosen as the smallest value for which BRR fits in 0..255.

use crate::clock::Clocks;
use crate::gpio::{Alternate, Input, Pin};
use ra4m1::{SCI2, SCI9};

/// PSEL value that assigns SCI9 asynchronous serial to P109/P110.
const PSEL_SCI: u8 = 0b0_0101;

/// Receive error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// Overrun (receive buffer overwritten).
    Overrun,
    /// Framing error (invalid stop bit).
    Framing,
    /// Parity error.
    Parity,
}

impl embedded_io::Error for Error {
    fn kind(&self) -> embedded_io::ErrorKind {
        embedded_io::ErrorKind::Other
    }
}

impl embedded_hal_nb::serial::Error for Error {
    fn kind(&self) -> embedded_hal_nb::serial::ErrorKind {
        match self {
            Error::Overrun => embedded_hal_nb::serial::ErrorKind::Overrun,
            Error::Framing => embedded_hal_nb::serial::ErrorKind::FrameFormat,
            Error::Parity => embedded_hal_nb::serial::ErrorKind::Parity,
        }
    }
}

/// An invalid or insufficiently accurate baud-rate request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidBaudRate;

/// Common interface for the RA4M1 SCI peripherals used by Serial.
pub trait Sci {
    fn enable();
    fn configure(&self, brr: u8, cks: u8);
    fn enable_tx_rx(&self);
    fn disable(&self);
    fn tx_ready(&self) -> bool;
    fn write_byte(&self, byte: u8);
    fn tx_done(&self) -> bool;
    fn rx_status(&self) -> Result<bool, Error>;
    fn read_byte(&self) -> u8;
}

/// UART backed by an RA4M1 SCI peripheral.
pub struct Serial<S, TX, RX> {
    sci: S,
    _tx: TX,
    _rx: RX,
}

/// Finds the smallest `n` for which BRR fits in 0..=255, given `divisor = 8 << (2n)`
/// (ABCS=1, BGDM=1), and returns `(BRR, CKS)`.
const fn calc_brr(pclka: u32, baud: u32) -> Option<(u8, u8)> {
    if baud == 0 {
        return None;
    }

    let mut n = 0u32;

    while n <= 3 {
        let div = 8u64 << (2 * n);
        let denom = div * baud as u64;

        // Round (N+1) to the nearest integer.
        let val = (pclka as u64 + denom / 2) / denom;

        if val >= 1 && val <= 256 {
            // Reject settings whose baud-rate error exceeds 3%.
            let target = denom * val;
            let error = (pclka as u64).abs_diff(target);

            if error * 100 <= target * 3 {
                return Some(((val - 1) as u8, n as u8));
            }

            return None;
        }

        n += 1;
    }

    None
}

impl Sci for SCI2 {
    fn enable() {
        critical_section::with(|_| unsafe {
            let system = &*ra4m1::SYSTEM::PTR;
            let mstp = &*ra4m1::MSTP::PTR;

            // Unlock write protection for low-power/module-stop registers (PRC1).
            system.prcr.write(|w| w.bits(0xA502));
            mstp.mstpcrb.modify(|_, w| w.mstpb29()._0());
            system.prcr.write(|w| w.bits(0xA500));
        });
    }

    fn configure(&self, brr: u8, cks: u8) {
        // Stop TX/RX while configuring.
        self.scr().write(|w| unsafe { w.bits(0) });

        // Restore all format-related registers that a bootloader may have changed.
        self.smr().reset();
        self.scmr.reset();
        self.semr.reset();
        self.snfr.reset();

        // SMR: asynchronous, 8-bit, no parity, 1 stop bit, CKS=cks.
        self.smr().write(|w| unsafe { w.bits(cks) });

        // SEMR: ABCS=1, BGDM=1 (improves baud rate resolution).
        self.semr.write(|w| w.abcs()._1().bgdm()._1());

        self.brr.write(|w| unsafe { w.bits(brr) });

        // Clear receive error flags.
        self.ssr()
            .modify(|_, w| w.orer()._0().fer()._0().per()._0());
    }

    fn enable_tx_rx(&self) {
        self.scr().modify(|_, w| w.te()._1().re()._1());
    }

    fn disable(&self) {
        self.scr().write(|w| unsafe { w.bits(0) });
    }

    fn tx_ready(&self) -> bool {
        self.ssr().read().tdre().is_1()
    }

    fn write_byte(&self, byte: u8) {
        self.tdr.write(|w| unsafe { w.bits(byte) });
    }

    fn tx_done(&self) -> bool {
        self.ssr().read().tend().is_1()
    }

    fn rx_status(&self) -> Result<bool, Error> {
        let ssr = self.ssr().read();

        if ssr.orer().is_1() || ssr.fer().is_1() || ssr.per().is_1() {
            let err = if ssr.orer().is_1() {
                Error::Overrun
            } else if ssr.fer().is_1() {
                Error::Framing
            } else {
                Error::Parity
            };

            self.ssr()
                .modify(|_, w| w.orer()._0().fer()._0().per()._0());

            return Err(err);
        }

        Ok(ssr.rdrf().is_1())
    }

    fn read_byte(&self) -> u8 {
        self.rdr.read().bits()
    }
}

impl Sci for SCI9 {
    fn enable() {
        critical_section::with(|_| unsafe {
            let system = &*ra4m1::SYSTEM::PTR;
            let mstp = &*ra4m1::MSTP::PTR;

            // Unlock write protection for low-power/module-stop registers (PRC1).
            system.prcr.write(|w| w.bits(0xA502));
            mstp.mstpcrb.modify(|_, w| w.mstpb22()._0());
            system.prcr.write(|w| w.bits(0xA500));
        });
    }

    fn configure(&self, brr: u8, cks: u8) {
        // Stop TX/RX while configuring.
        self.scr().write(|w| unsafe { w.bits(0) });

        // Restore all format-related registers that a bootloader may have changed.
        self.smr().reset();
        self.scmr.reset();
        self.semr.reset();
        self.snfr.reset();

        // SMR: asynchronous, 8-bit, no parity, 1 stop bit, CKS=cks.
        self.smr().write(|w| unsafe { w.bits(cks) });

        // SEMR: ABCS=1, BGDM=1 (improves baud rate resolution).
        self.semr.write(|w| w.abcs()._1().bgdm()._1());

        self.brr.write(|w| unsafe { w.bits(brr) });

        // Clear receive error flags.
        self.ssr()
            .modify(|_, w| w.orer()._0().fer()._0().per()._0());
    }

    fn enable_tx_rx(&self) {
        self.scr().modify(|_, w| w.te()._1().re()._1());
    }

    fn disable(&self) {
        self.scr().write(|w| unsafe { w.bits(0) });
    }

    fn tx_ready(&self) -> bool {
        self.ssr().read().tdre().is_1()
    }

    fn write_byte(&self, byte: u8) {
        self.tdr.write(|w| unsafe { w.bits(byte) });
    }

    fn tx_done(&self) -> bool {
        self.ssr().read().tend().is_1()
    }

    fn rx_status(&self) -> Result<bool, Error> {
        let ssr = self.ssr().read();

        if ssr.orer().is_1() || ssr.fer().is_1() || ssr.per().is_1() {
            let err = if ssr.orer().is_1() {
                Error::Overrun
            } else if ssr.fer().is_1() {
                Error::Framing
            } else {
                Error::Parity
            };

            self.ssr()
                .modify(|_, w| w.orer()._0().fer()._0().per()._0());

            return Err(err);
        }

        Ok(ssr.rdrf().is_1())
    }

    fn read_byte(&self) -> u8 {
        self.rdr.read().bits()
    }
}

impl<S: Sci, TX, RX> Serial<S, TX, RX> {
    fn configure<
        M1,
        M2,
        const TX_PORT: char,
        const TX_PIN: u8,
        const RX_PORT: char,
        const RX_PIN: u8,
    >(
        sci: S,
        tx: Pin<TX_PORT, TX_PIN, M1>,
        rx: Pin<RX_PORT, RX_PIN, M2>,
        baud: u32,
        clocks: &Clocks,
    ) -> Result<
        Serial<S, Pin<TX_PORT, TX_PIN, Alternate>, Pin<RX_PORT, RX_PIN, Alternate>>,
        InvalidBaudRate,
    > {
        let (brr, cks) = calc_brr(clocks.pclka().to_Hz(), baud).ok_or(InvalidBaudRate)?;

        S::enable();

        // Assign the pins to the SCI function.
        let _tx = tx.into_alternate(PSEL_SCI);
        let _rx = rx.into_alternate(PSEL_SCI);

        // Configure the SCI for asynchronous 8N1 operation.
        sci.configure(brr, cks);

        // The manual requires waiting at least one bit time after setting BRR before
        // enabling TX/RX.
        cortex_m::asm::delay(clocks.sysclk().to_Hz() / baud + 64);

        // Enable TX/RX (polling only, no interrupts).
        sci.enable_tx_rx();

        Ok(Serial { sci, _tx, _rx })
    }

    /// Releases the SCI handle and pins.
    pub fn release(self) -> (S, TX, RX) {
        self.sci.disable();
        (self.sci, self._tx, self._rx)
    }

    /// Sends one byte (waits for the transmit data register to be empty).
    pub fn write_byte(&mut self, byte: u8) -> nb::Result<(), core::convert::Infallible> {
        if !self.sci.tx_ready() {
            return Err(nb::Error::WouldBlock);
        }

        self.sci.write_byte(byte);
        Ok(())
    }

    /// Waits for the transmission to complete (TEND).
    pub fn flush(&mut self) -> nb::Result<(), core::convert::Infallible> {
        if !self.sci.tx_done() {
            return Err(nb::Error::WouldBlock);
        }

        Ok(())
    }

    /// Receives one byte (waits for reception to complete). Clears error flags before
    /// returning them.
    pub fn read_byte(&mut self) -> nb::Result<u8, Error> {
        let ready = self.sci.rx_status().map_err(nb::Error::Other)?;

        if !ready {
            return Err(nb::Error::WouldBlock);
        }

        Ok(self.sci.read_byte())
    }
}

/// SCI2 UART on D1/P302 (TX) and D0/P301 (RX).
impl Serial<SCI2, Pin<'3', 2, Alternate>, Pin<'3', 1, Alternate>> {
    /// Consumes SCI2 and the TX(D1=P302)/RX(D0=P301) pins, and builds a UART at `baud` bps.
    pub fn new<M1, M2>(
        sci: SCI2,
        tx: Pin<'3', 2, M1>,
        rx: Pin<'3', 1, M2>,
        baud: u32,
        clocks: &Clocks,
    ) -> Result<Serial<SCI2, Pin<'3', 2, Alternate>, Pin<'3', 1, Alternate>>, InvalidBaudRate> {
        Self::configure(sci, tx, rx, baud, clocks)
    }
}

/// SCI9 UART on P109 (TX) and P110 (RX), used by the UNO R4 WiFi USB bridge.
impl Serial<SCI9, Pin<'1', 9, Alternate>, Pin<'1', 10, Alternate>> {
    /// Consumes SCI9 and configures the P109/P110 USB bridge UART at `baud` bps.
    pub fn new_sci9(
        sci: SCI9,
        baud: u32,
        clocks: &Clocks,
    ) -> Result<Serial<SCI9, Pin<'1', 9, Alternate>, Pin<'1', 10, Alternate>>, InvalidBaudRate>
    {
        let tx = Pin::<'1', 9, Input>::new();
        let rx = Pin::<'1', 10, Input>::new();

        Self::configure(sci, tx, rx, baud, clocks)
    }
}

// ===== embedded-hal-nb =====

impl<S: Sci, TX, RX> embedded_hal_nb::serial::ErrorType for Serial<S, TX, RX> {
    type Error = Error;
}

impl<S: Sci, TX, RX> embedded_hal_nb::serial::Write<u8> for Serial<S, TX, RX> {
    fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        self.write_byte(word).map_err(|e| match e {
            nb::Error::WouldBlock => nb::Error::WouldBlock,
            nb::Error::Other(_) => unreachable!(),
        })
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        Serial::flush(self).map_err(|e| match e {
            nb::Error::WouldBlock => nb::Error::WouldBlock,
            nb::Error::Other(_) => unreachable!(),
        })
    }
}

impl<S: Sci, TX, RX> embedded_hal_nb::serial::Read<u8> for Serial<S, TX, RX> {
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        self.read_byte()
    }
}

// ===== embedded-io =====

impl<S: Sci, TX, RX> embedded_io::ErrorType for Serial<S, TX, RX> {
    type Error = Error;
}

impl<S: Sci, TX, RX> embedded_io::Write for Serial<S, TX, RX> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        for &b in buf {
            nb::block!(self.write_byte(b)).ok();
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        nb::block!(Serial::flush(self)).ok();
        Ok(())
    }
}

impl<S: Sci, TX, RX> embedded_io::Read for Serial<S, TX, RX> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        if buf.is_empty() {
            return Ok(0);
        }

        buf[0] = nb::block!(self.read_byte())?;
        Ok(1)
    }
}

// ===== core::fmt::Write (for write!/writeln!) =====

impl<S: Sci, TX, RX> core::fmt::Write for Serial<S, TX, RX> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.bytes() {
            nb::block!(self.write_byte(b)).ok();
        }

        Ok(())
    }
}
