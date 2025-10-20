use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use em2rs::Em2rsState;
use utilities::moving_average::MovingAverage;

use crate::command_executor::{
    motor::command_sender::Em2rsCommandSender, sensors::command_sender::SensorsCommandSender,
};

pub struct MoveThread {
    axis: u8,

    m7015_cs: SensorsCommandSender,
    em2rs_cs: Em2rsCommandSender,

    target_position: f32,
    position_window: f32,
    time_limit: Duration,

    filter: MovingAverage,

    moving: Arc<AtomicBool>,
    start_time: Instant,
    steps_per_mm: u32,
}

impl MoveThread {
    pub fn new(
        axis: u8,
        m7015_cs: SensorsCommandSender,
        em2rs_cs: Em2rsCommandSender,
        target_position: f32,
        position_window: f32,
        time_limit: Duration,
        moving: Arc<AtomicBool>,
        steps_per_mm: u32,
    ) -> Self {
        Self {
            axis,

            m7015_cs,
            em2rs_cs,

            filter: MovingAverage::new(20),

            target_position,
            position_window,
            time_limit,

            moving,
            start_time: Instant::now(),
            steps_per_mm,
        }
    }

    async fn position(&self) -> std::io::Result<f32> {
        self.m7015_cs.read_position(self.axis).await
    }

    fn is_moving(&self) -> bool {
        self.moving.load(Ordering::SeqCst)
    }

    fn time_limit_exceeded(&self) -> bool {
        self.start_time.elapsed() > self.time_limit
    }

    async fn get_state(&self) -> std::io::Result<Em2rsState> {
        self.em2rs_cs.get_state().await
    }

    async fn stop(&self) -> std::io::Result<()> {
        self.em2rs_cs.stop().await
    }

    async fn send_steps(&self, steps: i32) -> std::io::Result<()> {
        self.em2rs_cs.send_steps(steps).await
    }

    async fn move_relative(&self, error: f32) -> std::io::Result<()> {
        let steps = (error * self.steps_per_mm as f32).round() as i32;

        self.send_steps(steps).await?;

        while self.is_moving() && self.get_state().await?.is_moving() && !self.time_limit_exceeded()
        {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        Ok(())
    }

    pub async fn run(&mut self) -> std::io::Result<()> {
        while self.is_moving() && !self.time_limit_exceeded() {
            let current_position = self.position().await?;

            let error = current_position - self.target_position;
            self.filter.add(error);

            if self.filter.get_rms() <= self.position_window {
                break;
            }

            self.move_relative(error).await?;

            let state = self.get_state().await?;

            if state.high_limit_triggered() && error > 0.0 {
                break;
            } else if state.low_limit_triggered() && error < 0.0 {
                break;
            } else {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }

        Ok(())
    }
}

impl Drop for MoveThread {
    fn drop(&mut self) {
        self.moving.store(false, Ordering::SeqCst);
    }
}
