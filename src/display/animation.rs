use super::*;

impl<P, const BLEVELS: u8> LedMatrix<P, 5, 5, BLEVELS>
where
    P: OutputPin,
{
    /// Scroll the provided text across the LED display using default duration based on text length
    pub async fn scroll(&mut self, text: &str) {
        self.scroll_with_speed(text, Duration::from_secs((text.len() / 2) as u64))
            .await;
    }

    /// Scroll the provided text across the screen within the provided duration
    pub async fn scroll_with_speed(&mut self, text: &str, speed: Duration) {
        self.animate(text.as_bytes(), AnimationEffect::Slide, speed).await;
    }

    /// Apply animation based on data with the given effect during the provided duration
    pub async fn animate(&mut self, data: &[u8], effect: AnimationEffect, duration: Duration) {
        let mut animation: Animation<'_, 5, 5> =
            Animation::new(AnimationData::Bytes(data), effect, duration).unwrap();
        loop {
            match animation.next(Instant::now()) {
                AnimationState::Apply(f) => {
                    self.with_frame_buffer(|fb| *fb = f);
                }
                AnimationState::Wait => {}
                AnimationState::Done => {
                    break;
                }
            }
            self.render();
            Timer::after(REFRESH_INTERVAL).await;
        }
        self.clear();
    }

    /// Animate a slice of frames using the provided effect during the provided duration
    pub async fn animate_frames(&mut self, data: &[Frame<5, 5>], effect: AnimationEffect, duration: Duration) {
        let mut animation: Animation<'_, 5, 5> =
            Animation::new(AnimationData::Frames(data), effect, duration).unwrap();
        loop {
            match animation.next(Instant::now()) {
                AnimationState::Apply(f) => {
                    self.with_frame_buffer(|fb| *fb = f);
                }
                AnimationState::Wait => {}
                AnimationState::Done => {
                    break;
                }
            }
            self.render();
            Timer::after(REFRESH_INTERVAL).await;
        }
        self.clear();
    }
}

/// An effect filter to apply for an animation
#[derive(Clone, Copy)]
pub enum AnimationEffect {
    /// No effect
    None,
    /// Sliding effect
    Slide,
}

enum AnimationData<'a, const XSIZE: usize, const YSIZE: usize> {
    Frames(&'a [Frame<XSIZE, YSIZE>]),
    Bytes(&'a [u8]),
}

impl<'a, const XSIZE: usize, const YSIZE: usize> AnimationData<'a, XSIZE, YSIZE> {
    fn len(&self) -> usize {
        match self {
            AnimationData::Frames(f) => f.len(),
            AnimationData::Bytes(f) => f.len(),
        }
    }
}

impl<'a, const XSIZE: usize, const YSIZE: usize> AnimationData<'a, XSIZE, YSIZE> {
    fn frame(&self, idx: usize) -> Frame<XSIZE, YSIZE> {
        match self {
            AnimationData::Frames(f) => f[idx],
            AnimationData::Bytes(f) => fonts::CharFrame::try_5x5(fonts::ascii_frame(f[idx])).unwrap(),
        }
    }
}

struct Animation<'a, const XSIZE: usize, const YSIZE: usize> {
    frames: AnimationData<'a, XSIZE, YSIZE>,
    sequence: usize,
    frame_index: usize,
    index: usize,
    length: usize,
    effect: AnimationEffect,
    wait: Duration,
    next: Instant,
}

#[derive(PartialEq, Debug)]
enum AnimationState<const XSIZE: usize, const YSIZE: usize> {
    Wait,
    Apply(Frame<XSIZE, YSIZE>),
    Done,
}

impl<'a, const XSIZE: usize, const YSIZE: usize> Animation<'a, XSIZE, YSIZE> {
    pub fn new(
        frames: AnimationData<'a, XSIZE, YSIZE>,
        effect: AnimationEffect,
        duration: Duration,
    ) -> Result<Self, AnimationError> {
        if frames.len() == 0 {
            return Err(AnimationError::NoFrames);
        };
        let length = match effect {
            AnimationEffect::Slide => frames.len() * XSIZE,
            AnimationEffect::None => frames.len(),
        };

        if let Some(wait) = duration.checked_div(length as u32) {
            Ok(Self {
                frames,
                frame_index: 0,
                sequence: 0,
                index: 0,
                length,
                effect,
                wait,
                next: Instant::now(),
            })
        } else {
            Err(AnimationError::TooFast)
        }
    }

    fn shift(frame: &mut Frame<XSIZE, YSIZE>, shift: isize) {
        if shift == 0 {
            return;
        }
        let (shift_left, shift) = if shift < 0 {
            (true, -shift as usize)
        } else {
            assert!(shift > 0);
            (false, shift as usize)
        };
        for row in frame {
            if shift_left {
                for cid in 0..XSIZE - shift {
                    row[cid] = row[cid + shift];
                }
                for cell in &mut row[XSIZE - shift..XSIZE] {
                    *cell = 0;
                }
            } else {
                for cid in XSIZE - shift..XSIZE {
                    row[cid] = row[cid - shift];
                }
                for cell in &mut row[0..XSIZE - shift] {
                    *cell = 0;
                }
            }
        }
    }

    fn merge(frame: &mut Frame<XSIZE, YSIZE>, source: &Frame<XSIZE, YSIZE>) {
        for (dst, src) in frame.iter_mut().zip(source.iter()) {
            for (dst, src) in dst.iter_mut().zip(src.iter()) {
                *dst = (*dst).max(*src);
            }
        }
    }

    fn current(&self) -> Frame<XSIZE, YSIZE> {
        let mut current = self.frames.frame(self.frame_index);

        let mut next = if self.frame_index < self.frames.len() - 1 {
            self.frames.frame(self.frame_index + 1)
        } else {
            [[0; XSIZE]; YSIZE]
        };

        Self::shift(&mut current, -(self.sequence as isize));
        Self::shift(&mut next, (XSIZE - self.sequence) as isize);
        Self::merge(&mut current, &next);
        current
    }

    fn next(&mut self, now: Instant) -> AnimationState<XSIZE, YSIZE> {
        if self.next <= now {
            if self.index < self.length {
                let current = self.current();
                if self.sequence >= XSIZE - 1 {
                    self.sequence = match self.effect {
                        AnimationEffect::None => XSIZE,
                        AnimationEffect::Slide => 0,
                    };
                    self.frame_index += 1;
                } else {
                    self.sequence += 1;
                }

                self.index += 1;
                self.next += self.wait;
                AnimationState::Apply(current)
            } else {
                AnimationState::Done
            }
        } else {
            AnimationState::Wait
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Errors produced when running animations
pub enum AnimationError {
    /// No animation frames were supplied.
    NoFrames,
    /// Animation scroll is too fast to keep up with the refresh rate
    TooFast,
}
