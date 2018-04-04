use time::SteadyTime;

pub struct Timer {
    start: SteadyTime,
    last_frame: SteadyTime,
}

pub fn new() -> Box<Timer> {
    Box::new(Timer {
        start: SteadyTime::now(),
        last_frame: SteadyTime::now(),
    })
}

impl Timer {
    pub fn elapsed(&self) -> i64 {
        let dur = SteadyTime::now() - self.start;
        dur.num_milliseconds()
    }

    pub fn frame_time(&mut self) -> i64 {
        let origin = self.last_frame;
        self.last_frame = SteadyTime::now();
        (self.last_frame - origin).num_milliseconds()
    }
}
