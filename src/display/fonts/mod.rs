//! Bitmaps and fonts for the micro:bit

use super::Frame;
mod pendolino;

mod bitmaps {
    use super::*;

    #[rustfmt::skip]
    /// A check-mark bitmap
    pub const CHECK_MARK: Frame<5, 5> = frame_5x5(&[
        0b00000,
        0b00001,
        0b00010,
        0b10100,
        0b01000,
    ]);

    #[rustfmt::skip]
    /// A cross-mark bitmap
    pub const CROSS_MARK: Frame<5, 5> = frame_5x5(&[
        0b00000,
        0b01010,
        0b00100,
        0b01010,
        0b00000,
    ]);

    #[rustfmt::skip]
    /// A left arrow bitmap
    pub const ARROW_LEFT: Frame<5, 5> = frame_5x5(&[
        0b00100,
        0b01000,
        0b11111,
        0b01000,
        0b00100,
    ]);

    #[rustfmt::skip]
    /// A right arrow bitmap
    pub const ARROW_RIGHT: Frame<5, 5> = frame_5x5(&[
        0b00100,
        0b00010,
        0b11111,
        0b00010,
        0b00100,
    ]);

    /// Construct a 5×5 frame from a byte slice.
    /// Each byte is a column bitmap.
    pub const fn frame_5x5(input: &[u8; 5]) -> Frame<5, 5> {
        let mut frame: Frame<5, 5> = [[0; 5]; 5];
        let mut rid = 0;
        let mut cid = 0;
        loop {
            frame[rid][cid] = (input[rid] >> (4 - cid)) & 1;
            cid += 1;
            if cid >= 5 {
                cid = 0;
                rid += 1;
            }
            if rid >= 5 {
                break;
            }
        }
        frame
    }
}

pub use bitmaps::*;

/// Specialized [Frame] for 5×5 display.
pub struct CharFrame(Frame<5, 5>);

/// Return a frame whose bitmap is an ASCII character.
// XXX FIXME: should panic for non-ASCII u8. As it
// is, it will produce some weird iso-8859-1 variant
// when the high bit is set. See the docs for the
// `From<u8>` impl for `char`.
pub fn ascii_frame(ascii: u8) -> CharFrame {
    char_frame(char::from(ascii))
}

/// Return a frame whose bitmap is a Unicode character.
// XXX FIXME: currently silently turns all
//     non-pendolino-printable characters into blanks.
// XXX FIXME: use special characters defined above.
pub fn char_frame(c: char) -> CharFrame {
    let n = c as usize;
    if n > pendolino::PRINTABLE_START && n < pendolino::PRINTABLE_START + pendolino::PRINTABLE_COUNT {
        CharFrame(frame_5x5(&pendolino::PENDOLINO3[n - pendolino::PRINTABLE_START]))
    } else {
        CharFrame(frame_5x5(&[0, 0, 0, 0, 0]))
    }
}

impl CharFrame {
    /// Return `self` as a 5×5 [Frame] in a 5×5 context, and
    /// an error otherwise.
    pub fn try_5x5<const XSIZE: usize, const YSIZE: usize>(self) -> Result<Frame<XSIZE, YSIZE>, FontError> {
        if XSIZE == 5 && YSIZE == 5 {
            // XXX This is disgusting. Hopefully the compiler will
            // remove the gratuitous copy code.
            let mut result = [[0; XSIZE]; YSIZE];
            for (src, dst) in self.0.iter().zip(result.iter_mut()) {
                for (src, dst) in src.iter().zip(dst.iter_mut()) {
                    *dst = *src;
                }
            }
            return Ok(result);
        }
        Err(FontError::BadDimension)
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Errors produced when working with fonts.
pub enum FontError {
    /// Requested unavailable font dimension.
    BadDimension,
}
