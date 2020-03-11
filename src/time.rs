use std::{
    thread,
    time::{Duration, Instant},
};

#[derive(Debug)]
pub struct Time {
    start: Instant,
    frame_count: u32,
    fps: u32,
}

impl Time {
    pub fn new(fps: u32) -> Self {
        Self {
            start: Instant::now(),
            frame_count: 0,
            fps,
        }
    }

    /// restarts this timer, useful after loading
    /// a new scene
    pub fn restart(&mut self) {
        self.frame_count = 0;
        self.start = Instant::now();
    }

    pub fn framerate(&self) -> u32 {
        self.fps
    }

    pub fn frame(&mut self) {
        if self.fps != 0 {
            self.frame_count += 1;
            let finish = Duration::from_micros(1_000_000 / u64::from(self.fps)) * self.frame_count;
            if self.start.elapsed() < finish {
                while self.start.elapsed() < finish {
                    thread::yield_now();
                }
            } else {
                // FIXME: we currently just skip lag frames
                // add a way for users to modify this behavior.
                warn!(
                    "Lag: {} ms",
                    (self.start.elapsed() - finish).as_secs_f32() / 1000.0
                );
                self.restart();
            }
        }
    }
}
