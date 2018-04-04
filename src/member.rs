use std::net::SocketAddr;
use cgmath::{Point2, Vector2};

use timers;

pub struct Member {
    pub addr: SocketAddr,
    pub session_on: bool,
    pub timer: Box<timers::Timer>,
    pub gpos: Point2<f32>,
    pub moving: bool,
    pub move_direction: Vector2<f32>,
    pub debug_move_start: i64,
    pub debug_move_stop: i64,
}

impl Member {
    pub fn default(addr: SocketAddr) -> Member {
        Member {
            addr: addr,
            session_on: false,
            timer: timers::new(),
            gpos: Point2::<f32> { x: 0f32, y: 0f32 },
            moving: false,
            move_direction: Vector2::<f32> { x: 0f32, y: 0f32 },
            debug_move_start: 0,
            debug_move_stop: 0,
        }
    }

    pub fn update(&mut self) {
        let dt = self.timer.frame_time();

        if self.moving {
            self.gpos += self.move_direction * dt as f32;
            // println!("{:?}", self.gpos);
        }
    }
}
