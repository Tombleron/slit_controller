use crate::command_executor::{
    motor::command_sender::Em2rsCommandSender, sensors::command_sender::SensorsCommandSender,
};
use em2rs::StateParams;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use utilities::{
    motor_controller::{Motor, MotorState},
    moving_average::MovingAverage,
};

pub struct MoveThread {
    axis: usize,

    sensors_cs: SensorsCommandSender,
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
        axis: usize,
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

            sensors_cs: m7015_cs,
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

    async fn stop(&self) -> std::io::Result<()> {
        self.em2rs_cs.stop(self.axis).await
    }

    async fn send_steps(&self, steps: i32) -> std::io::Result<()> {
        self.em2rs_cs.send_steps(self.axis, steps).await
    }
}

impl Drop for MoveThread {
    fn drop(&mut self) {
        self.moving.store(false, Ordering::SeqCst);
        dbg!("Move thread dropped");
    }
}

impl Motor for MoveThread {
    async fn position(&self) -> Result<f32, String> {
        self.sensors_cs
            .get_position(self.axis as u8)
            .await
            .map_err(|err| err.to_string())
    }

    async fn state(&self) -> Result<impl MotorState, String> {
        self.em2rs_cs
            .get_state(self.axis)
            .await
            .map(|state| Em2rsState(state))
            .map_err(|err| err.to_string())
    }

    async fn move_relative(&mut self, error: f32) -> Result<(), String> {
        let error = -error;
        let steps = if error.abs() == 0.0 {
            0
        } else if error.abs() < 0.001 {
            if error > 0.0 { 10 } else { -10 }
        } else {
            (error * self.steps_per_mm as f32) as i32
        };
        dbg!(steps);

        let _result = self
            .send_steps(steps)
            .await
            .map_err(|e| format!("Failed to move relative: {}", e))?;

        while self.is_moving() && self.state().await?.is_moving() && !self.is_time_limit_exceeded()
        {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        Ok(())
    }

    fn get_position_window(&self) -> f32 {
        self.position_window
    }

    fn get_time_limit(&self) -> Duration {
        self.time_limit
    }

    fn get_start_time(&self) -> Instant {
        self.start_time
    }

    fn get_target_position(&self) -> f32 {
        self.target_position
    }

    fn add_error(&mut self, error: f32) {
        self.filter.add(error);
    }

    fn get_rms(&self) -> f32 {
        self.filter.get_rms()
    }

    fn is_moving(&self) -> bool {
        self.moving.load(Ordering::Relaxed)
    }

    fn set_moving(&mut self, is_moving: bool) {
        self.moving.store(is_moving, Ordering::Relaxed);
    }
}

struct Em2rsState(StateParams);

impl MotorState for Em2rsState {
    fn start_switch(&self) -> bool {
        self.0.low_limit_triggered()
    }

    fn end_switch(&self) -> bool {
        self.0.high_limit_triggered()
    }

    fn is_moving(&self) -> bool {
        self.0.is_moving()
    }
}
