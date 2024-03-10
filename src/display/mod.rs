//! Driver a NxM LED matrix display
//!
//! * Can display 5x5 bitmaps from raw data or characters
//! * Methods for scrolling text across LED matrix or displaying a bitmap for a duration
use embassy_time::{block_for, Duration, Instant, Timer};
use embedded_hal::digital::OutputPin;

mod brightness;
pub mod fonts;
mod animation;

pub use brightness::*;
pub use animation::*;

/// Internally, a `Frame` is a 2D array of bytes
/// representing brightness levels.
pub type Frame<const COLS: usize, const ROWS: usize> = [[u8; COLS]; ROWS];

const REFRESH_INTERVAL: Duration = Duration::from_micros(500);

/// Led matrix driver supporting arbitrary sized led matrixes.
pub struct LedMatrix<P, const ROWS: usize, const COLS: usize, const BLEVELS: u8>
where
    P: OutputPin + 'static,
{
    pin_rows: [P; ROWS],
    pin_cols: [P; COLS],
    frame_buffer: Frame<COLS, ROWS>,
    row_p: usize,
    brightness: Brightness<BLEVELS>,
}

impl<P, const ROWS: usize, const COLS: usize, const BLEVELS: u8> LedMatrix<P, ROWS, COLS, BLEVELS>
where
    P: OutputPin,
{
    /// Create a new instance of an LED matrix using the provided pins.
    pub fn new(pin_rows: [P; ROWS], pin_cols: [P; COLS]) -> Self {
        LedMatrix {
            pin_rows,
            pin_cols,
            frame_buffer: [[0; COLS]; ROWS],
            row_p: 0,
            brightness: Default::default(),
        }
    }

    /// Clear all LEDs.
    pub fn clear(&mut self) {
        self.frame_buffer = [[0; COLS]; ROWS];
        for row in self.pin_rows.iter_mut() {
            row.set_high().ok();
        }

        for col in self.pin_cols.iter_mut() {
            col.set_high().ok();
        }
    }

    /// Set point (x,y) in the frame buffer to value.
    /// XXX FIXME: check ranges.
    pub fn set(&mut self, x: usize, y: usize, level: u8) {
        self.frame_buffer[x][y] = level;
    }

    /// Access/modify the frame buffer.
    pub fn with_frame_buffer<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Frame<COLS, ROWS>) -> R,
    {
        f(&mut self.frame_buffer)
    }

    /// Adjust the brightness level
    pub fn set_brightness(&mut self, brightness: Brightness<BLEVELS>) {
        self.brightness = brightness;
    }

    /// Increase brightness relative to current setting
    pub fn increase_brightness(&mut self) {
        self.brightness += 1;
    }

    /// Decrease brightness relative to current setting
    pub fn decrease_brightness(&mut self) {
        self.brightness -= 1;
    }

    /// Perform a full refresh of the display based on the current frame buffer
    pub fn render(&mut self) {
        for row in self.pin_rows.iter_mut() {
            row.set_low().ok();
        }

        for (cid, col) in self.pin_cols.iter_mut().enumerate() {
            if self.frame_buffer[self.row_p][cid] > 0 {
                col.set_low().ok();
            } else {
                col.set_high().ok();
            }
        }

        self.pin_rows[self.row_p].set_high().ok();

        // Adjust interval will impact brightness of the LEDs
        block_for(Duration::from_micros(
            ((<Brightness<BLEVELS>>::MAX.level() - self.brightness.level()) as u64) * 6000
                / <Brightness<BLEVELS>>::MAX.level() as u64,
        ));

        self.pin_rows[self.row_p].set_low().ok();

        self.row_p = (self.row_p + 1) % ROWS;
    }

    /// Display the current frame for the duration. Handles screen refresh
    /// in an async display loop.
    pub async fn display(&mut self, length: Duration) {
        let end = Instant::now() + length;
        while Instant::now() < end {
            self.render();
            Timer::after(REFRESH_INTERVAL).await;
        }
        self.clear();
    }

    /// Display the current frame for the duration. Handles screen refresh
    /// in a blocking display loop.
    pub fn display_blocking(&mut self, length: Duration) {
        let end = Instant::now() + length;
        while Instant::now() < end {
            self.render();
            block_for(REFRESH_INTERVAL);
        }
        self.clear();
    }

    /// Disassemble the `LedMatrix` and return the pins, as
    /// an array of row pins and an array of column pins.
    pub fn into_inner(self) -> ([P; ROWS], [P;COLS]) {
        (self.pin_rows, self.pin_cols)
    }
}
