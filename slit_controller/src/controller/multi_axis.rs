use std::io;
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use standa::command::state::StateParams;
use tracing::{debug, error, info, instrument, warn};

use crate::controller::single_axis::SingleAxis;

pub struct MultiAxis {
    rf256_client: Option<Arc<Mutex<TcpStream>>>,
    standa_clients: [Option<Arc<Mutex<TcpStream>>>; 4],
    rf256_ids: [u8; 4],

    axes: [Option<SingleAxis<TcpStream>>; 4],

    config: MultiAxisConfig,
}

impl MultiAxis {
    #[instrument(
        skip(config),
        fields(
            rf256_ip = %config.rf256_ip,
            rf256_port = %config.rf256_port,
        )
    )]
    pub fn from_config(config: MultiAxisConfig) -> Self {
        info!("Initializing MultiAxis controller");
        // Empty clients for RF256 and Standa
        // They are initialized later, when needed client
        // is requested
        Self {
            rf256_client: None,
            standa_clients: [None, None, None, None],
            rf256_ids: [
                config.upper_standa_id,
                config.lower_standa_id,
                config.left_standa_id,
                config.right_standa_id,
            ],
            axes: [None, None, None, None],
            config,
        }
    }

    #[instrument(skip(self))]
    fn get_rf256_client(&mut self) -> io::Result<Arc<Mutex<TcpStream>>> {
        if self.rf256_client.is_none() {
            debug!("RF256 client is not initialized, creating a new one");

            debug!(
                "Connecting to RF256 at {}:{}",
                self.config.rf256_ip, self.config.rf256_port
            );
            match TcpStream::connect_timeout(
                &SocketAddr::new(
                    self.config.rf256_ip.parse().unwrap(),
                    self.config.rf256_port,
                ),
                Duration::from_secs(1),
            ) {
                Ok(stream) => {
                    if let Err(e) =
                        stream.set_read_timeout(Some(std::time::Duration::from_millis(100)))
                    {
                        warn!("Failed to set read timeout for RF256 client: {}", e);
                    }

                    debug!("Successfully connected to RF256");
                    self.rf256_client = Some(Arc::new(Mutex::new(stream)));
                }
                Err(e) => {
                    error!("Failed to connect to RF256: {}", e);
                    return Err(e);
                }
            }
        }
        Ok(self.rf256_client.as_ref().unwrap().clone())
    }

    #[instrument(skip(self))]
    fn get_standa_client(&mut self, index: usize) -> io::Result<Arc<Mutex<TcpStream>>> {
        if self.standa_clients[index].is_none() {
            debug!(
                "Standa client at index {} is not initialized, creating a new one",
                index
            );

            let ip = match index {
                0 => &self.config.upper_standa_ip,
                1 => &self.config.lower_standa_ip,
                2 => &self.config.left_standa_ip,
                3 => &self.config.right_standa_ip,
                _ => unreachable!(),
            };
            let port = match index {
                0 => self.config.upper_standa_port,
                1 => self.config.lower_standa_port,
                2 => self.config.left_standa_port,
                3 => self.config.right_standa_port,
                _ => unreachable!(),
            };

            debug!("Connecting to Standa at {}:{}", ip, port);
            match TcpStream::connect_timeout(
                &SocketAddr::new(ip.parse().unwrap(), port),
                Duration::from_secs(1),
            ) {
                Ok(stream) => {
                    if let Err(e) =
                        stream.set_read_timeout(Some(std::time::Duration::from_millis(100)))
                    {
                        warn!("Failed to set read timeout for Standa client: {}", e);
                    }

                    debug!("Successfully connected to Standa at index {}", index);
                    self.standa_clients[index] = Some(Arc::new(Mutex::new(stream)));
                }
                Err(e) => {
                    error!("Failed to connect to Standa at index {}: {}", index, e);
                    return Err(e);
                }
            }
        }

        Ok(self.standa_clients[index].as_ref().unwrap().clone())
    }

    #[instrument(skip(self))]
    fn reconnect_rf256_client(&mut self) -> io::Result<()> {
        if let Some(client) = &self.rf256_client {
            debug!(
                "Reconnecting to RF256 at {}:{}",
                self.config.rf256_ip, self.config.rf256_port
            );
            let mut stream = client.lock().unwrap();
            match TcpStream::connect_timeout(
                &SocketAddr::new(
                    self.config.rf256_ip.parse().unwrap(),
                    self.config.rf256_port,
                ),
                Duration::from_secs(1),
            ) {
                Ok(new_stream) => {
                    *stream = new_stream;
                    if let Err(e) =
                        stream.set_read_timeout(Some(std::time::Duration::from_millis(100)))
                    {
                        warn!("Failed to set read timeout for RF256 client: {}", e);
                        return Err(e);
                    }
                    debug!("Successfully reconnected to RF256");
                }
                Err(e) => {
                    error!("Failed to reconnect to RF256: {}", e);
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    #[instrument(skip(self))]
    fn reconnect_standa_client(&mut self, index: usize) -> io::Result<()> {
        if let Some(client) = &self.standa_clients[index] {
            let ip = match index {
                0 => &self.config.upper_standa_ip,
                1 => &self.config.lower_standa_ip,
                2 => &self.config.left_standa_ip,
                3 => &self.config.right_standa_ip,
                _ => unreachable!(),
            };
            let port = match index {
                0 => self.config.upper_standa_port,
                1 => self.config.lower_standa_port,
                2 => self.config.left_standa_port,
                3 => self.config.right_standa_port,
                _ => unreachable!(),
            };

            debug!(
                "Reconnecting to Standa at index {} ({}:{})",
                index, ip, port
            );
            let mut stream = client.lock().unwrap();
            match TcpStream::connect_timeout(
                &SocketAddr::new(ip.parse().unwrap(), port),
                Duration::from_secs(1),
            ) {
                Ok(new_stream) => {
                    *stream = new_stream;
                    if let Err(e) =
                        stream.set_read_timeout(Some(std::time::Duration::from_millis(100)))
                    {
                        warn!("Failed to set read timeout for Standa client: {}", e);
                        return Err(e);
                    }
                    debug!("Successfully reconnected to Standa at index {}", index);
                }
                Err(e) => {
                    error!("Failed to reconnect to Standa at index {}: {}", index, e);
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    #[instrument(skip(self))]
    fn get_axis(&mut self, index: usize) -> io::Result<&mut SingleAxis<TcpStream>> {
        if self.axes[index].is_none()
            || self.standa_clients[index].is_none()
            || self.rf256_client.is_none()
        {
            debug!(
                "Axis at index {} is not initialized, creating a new one",
                index
            );
            let rf256_client = self.get_rf256_client()?;
            let standa_client = self.get_standa_client(index)?;

            debug!("Creating new SingleAxis controller for index {}", index);
            self.axes[index] = Some(SingleAxis::new(
                rf256_client,
                self.rf256_ids[index],
                standa_client,
            ));
            debug!(
                "Successfully created SingleAxis controller for index {}",
                index
            );
        }

        Ok(self.axes[index].as_mut().unwrap())
    }

    #[instrument(skip(self))]
    pub fn get_velocity(&mut self, index: usize) -> io::Result<u32> {
        debug!("Getting velocity for axis {}", index);
        let axis = self.get_axis(index)?;
        let velocity = axis.get_velocity();
        debug!("Got velocity {} for axis {}", velocity, index);
        Ok(velocity)
    }

    #[instrument(skip(self))]
    pub fn set_velocity(&mut self, index: usize, velocity: u32) -> io::Result<()> {
        debug!("Setting velocity to {} for axis {}", velocity, index);
        let axis = self.get_axis(index)?;
        axis.set_velocity(velocity);
        debug!(
            "Successfully set velocity to {} for axis {}",
            velocity, index
        );
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn get_acceleration(&mut self, index: usize) -> io::Result<u16> {
        debug!("Getting acceleration for axis {}", index);
        let axis = self.get_axis(index)?;
        let acceleration = axis.get_acceleration();
        debug!("Got acceleration {} for axis {}", acceleration, index);
        Ok(acceleration)
    }

    #[instrument(skip(self))]
    pub fn set_acceleration(&mut self, index: usize, acceleration: u16) -> io::Result<()> {
        debug!(
            "Setting acceleration to {} for axis {}",
            acceleration, index
        );
        let axis = self.get_axis(index)?;
        axis.set_acceleration(acceleration);
        debug!(
            "Successfully set acceleration to {} for axis {}",
            acceleration, index
        );
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn get_deceleration(&mut self, index: usize) -> io::Result<u16> {
        debug!("Getting deceleration for axis {}", index);
        let axis = self.get_axis(index)?;
        let deceleration = axis.get_deceleration();
        debug!("Got deceleration {} for axis {}", deceleration, index);
        Ok(deceleration)
    }

    #[instrument(skip(self))]
    pub fn set_deceleration(&mut self, index: usize, deceleration: u16) -> io::Result<()> {
        debug!(
            "Setting deceleration to {} for axis {}",
            deceleration, index
        );
        let axis = self.get_axis(index)?;
        axis.set_deceleration(deceleration);
        debug!(
            "Successfully set deceleration to {} for axis {}",
            deceleration, index
        );
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn get_position_window(&mut self, index: usize) -> io::Result<f32> {
        debug!("Getting position window for axis {}", index);
        let axis = self.get_axis(index)?;
        let position_window = axis.get_position_window();
        debug!("Got position window {} for axis {}", position_window, index);
        Ok(position_window)
    }

    #[instrument(skip(self))]
    pub fn set_position_window(&mut self, index: usize, position_window: f32) -> io::Result<()> {
        debug!(
            "Setting position window to {} for axis {}",
            position_window, index
        );
        let axis = self.get_axis(index)?;
        axis.set_position_window(position_window);
        debug!(
            "Successfully set position window to {} for axis {}",
            position_window, index
        );
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn is_moving(&mut self, index: usize) -> bool {
        debug!("Checking if axis {} is moving", index);
        // FIXME: Probably this should return an error instead of false
        if let Ok(axis) = self.get_axis(index) {
            let moving = axis.is_moving();
            debug!("Axis {} is moving: {}", index, moving);
            moving
        } else {
            debug!(
                "Failed to get axis {}, returning false for is_moving",
                index
            );
            false
        }
    }

    #[instrument(skip(self))]
    pub fn move_to_position(&mut self, index: usize, position: f32) -> Result<(), String> {
        debug!("Moving axis {} to position {}", index, position);
        let axis = self.get_axis(index).map_err(|e| {
            error!("Failed to get axis {}: {}", index, e);
            e.to_string()
        })?;
        let result = axis.move_to_position(position);
        if let Err(ref e) = result {
            error!(
                "Failed to move axis {} to position {}: {}",
                index, position, e
            );
        } else {
            debug!(
                "Successfully started moving axis {} to position {}",
                index, position
            );
        }
        result
    }

    #[instrument(skip(self))]
    pub fn stop(&mut self, index: usize) -> Result<(), String> {
        debug!("Stopping axis {}", index);
        let axis = self.get_axis(index).map_err(|e| {
            error!("Failed to get axis {}: {}", index, e);
            e.to_string()
        })?;
        let result = axis.stop();
        if let Err(ref e) = result {
            error!("Failed to stop axis {}: {}", index, e);
        } else {
            debug!("Successfully stopped axis {}", index);
        }
        result
    }

    #[instrument(skip(self))]
    pub fn position(&mut self, index: usize) -> io::Result<f32> {
        debug!("Getting position for axis {}", index);
        let axis = self.get_axis(index)?;
        let result = axis.position();
        if let Err(ref e) = result {
            error!("Failed to get position for axis {}: {}", index, e);
            if e.kind() == io::ErrorKind::BrokenPipe {
                debug!("Detected broken pipe, attempting to reconnect RF256 client");
                self.reconnect_rf256_client()?;
            }
        } else if let Ok(pos) = result {
            debug!("Got position {} for axis {}", pos, index);
        }
        result
    }

    #[instrument(skip(self))]
    pub fn state(&mut self, index: usize) -> io::Result<StateParams> {
        debug!("Getting state for axis {}", index);
        let axis = self.get_axis(index)?;
        let result = axis.state();
        if let Err(ref e) = result {
            error!("Failed to get state for axis {}: {}", index, e);
            if e.kind() == io::ErrorKind::BrokenPipe || e.kind() == io::ErrorKind::WouldBlock {
                warn!("Detected broken pipe or would block, attempting to reconnect Standa client for index {}", index);
                self.reconnect_standa_client(index)?;
            }
        } else {
            debug!("Successfully got state for axis {}", index);
        }
        result
    }
}

pub struct MultiAxisConfig {
    pub rf256_ip: String,
    pub rf256_port: u16,

    pub upper_standa_ip: String,
    pub upper_standa_port: u16,
    pub upper_standa_id: u8,

    pub lower_standa_ip: String,
    pub lower_standa_port: u16,
    pub lower_standa_id: u8,

    pub left_standa_ip: String,
    pub left_standa_port: u16,
    pub left_standa_id: u8,

    pub right_standa_ip: String,
    pub right_standa_port: u16,
    pub right_standa_id: u8,
}

impl Default for MultiAxisConfig {
    fn default() -> Self {
        Self {
            rf256_ip: "192.168.0.51".to_string(),
            rf256_port: 60003,

            upper_standa_ip: "192.168.0.204".to_string(),
            upper_standa_port: 2000,
            upper_standa_id: 6,
            lower_standa_ip: "192.168.0.204".to_string(),
            lower_standa_port: 3000,
            lower_standa_id: 15,

            left_standa_ip: "192.168.0.205".to_string(),
            left_standa_port: 2000,
            left_standa_id: 5,

            right_standa_ip: "192.168.0.205".to_string(),
            right_standa_port: 3000,
            right_standa_id: 4,
        }
    }
}
