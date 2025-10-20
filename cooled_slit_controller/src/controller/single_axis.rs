use std::{
    io,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use em2rs::Em2rsState;
use icpcon::M7015;
use tokio::task::JoinHandle;

use crate::{
    command_executor::{
        motor::command_sender::Em2rsCommandSender, sensors::command_sender::SensorsCommandSender,
    },
    controller::move_thread::MoveThread,
};

pub struct MoveArgs {
    pub acceleration: u16,
    pub deceleration: u16,
    pub velocity: u16,
    pub position_window: f32,
    pub time_limit: Duration,
}

pub struct SingleAxis {
    axis: u8,

    m7015_cs: SensorsCommandSender,
    em2rs_cs: Em2rsCommandSender,

    move_thread: Option<JoinHandle<io::Result<()>>>,
    moving: Arc<AtomicBool>,

    steps_per_mm: u32,
}

impl SingleAxis {
    pub fn new(
        axis: u8,
        steps_per_mm: u32,
        m7015_cs: SensorsCommandSender,
        em2rs_cs: Em2rsCommandSender,
    ) -> Self {
        Self {
            axis,
            m7015_cs,
            em2rs_cs,
            move_thread: None,
            moving: Arc::new(AtomicBool::new(false)),
            steps_per_mm,
        }
    }

    pub fn get_m7015_cs(&self) -> SensorsCommandSender {
        self.m7015_cs.clone()
    }

    pub fn get_em2rs_cs(&self) -> Em2rsCommandSender {
        self.em2rs_cs.clone()
    }

    pub async fn position(&self) -> io::Result<f32> {
        self.m7015_cs.read_position(self.axis).await
    }

    pub async fn temperature(&self) -> io::Result<f32> {
        self.m7015_cs.read_temperature(self.axis).await
    }

    pub async fn state(&self) -> io::Result<Em2rsState> {
        self.em2rs_cs.get_state().await
    }

    pub fn is_moving(&self) -> bool {
        self.moving.load(Ordering::SeqCst)
    }

    pub async fn update_motor_settings(&self, args: &MoveArgs) -> io::Result<()> {
        self.em2rs_cs.set_acceleration(args.acceleration).await?;
        self.em2rs_cs.set_deceleration(args.deceleration).await?;
        self.em2rs_cs.set_velocity(args.velocity as u16).await?;
        Ok(())
    }

    pub async fn stop(&mut self) -> io::Result<()> {
        if self.is_moving() {
            self.em2rs_cs.stop().await?;

            if let Some(handle) = self.move_thread.take() {
                handle
                    .await?
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            }
            self.moving.store(false, Ordering::SeqCst);
        }
        Ok(())
    }

    pub async fn move_to(&mut self, target_position: f32, args: MoveArgs) -> io::Result<()> {
        if self.is_moving() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Axis is already moving",
            ));
        }

        self.update_motor_settings(&args).await?;

        self.moving.store(true, Ordering::SeqCst);

        let mut move_thread = MoveThread::new(
            self.axis,
            self.m7015_cs.clone(),
            self.em2rs_cs.clone(),
            target_position,
            args.position_window,
            args.time_limit,
            self.moving.clone(),
            self.steps_per_mm,
        );

        let handle = tokio::spawn(async move { move_thread.run().await });

        self.move_thread = Some(handle);

        Ok(())
    }
}
