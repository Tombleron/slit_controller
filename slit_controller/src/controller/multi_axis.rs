use std::io;
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use standa::command::state::StateParams;
use tracing::{debug, error, info, instrument, warn};

use crate::controller::single_axis::SingleAxis;

pub struct MultiAxis {
    rf256_client: Option<Arc<Mutex<TcpStream>>>,
    trid_client: Option<Arc<Mutex<TcpStream>>>,
    standa_clients: [Option<Arc<Mutex<TcpStream>>>; 4],

    rf256_ids: [u8; 4],
    trid_ids: [u8; 4],

    axes: [Option<SingleAxis<TcpStream>>; 4],

    config: MultiAxisConfig,
}

impl MultiAxis {
    pub fn from_config(config: MultiAxisConfig) -> Self {
        info!("Initializing MultiAxis controller");
        // Empty clients for RF256 and Standa
        // They are initialized later, when needed client
        // is requested
        Self {
            rf256_client: None,
            trid_client: None,
            standa_clients: [None, None, None, None],
            rf256_ids: [
                config.upper_rf256_id,
                config.lower_rf256_id,
                config.left_rf256_id,
                config.right_rf256_id,
            ],
            trid_ids: [
                config.upper_trid_id,
                config.lower_trid_id,
                config.left_trid_id,
                config.right_trid_id,
            ],
            axes: [None, None, None, None],
            config,
        }
    }

    fn get_trid_client(&mut self) -> io::Result<Arc<Mutex<TcpStream>>> {
        if self.trid_client.is_none() {
            debug!("TRID client is not initialized, creating a new one");

            debug!(
                "Connecting to TRID at {}:{}",
                self.config.trid_ip, self.config.trid_port
            );
            match TcpStream::connect_timeout(
                &SocketAddr::new(self.config.trid_ip.parse().unwrap(), self.config.trid_port),
                Duration::from_secs(1),
            ) {
                Ok(stream) => {
                    if let Err(e) =
                        stream.set_read_timeout(Some(std::time::Duration::from_millis(100)))
                    {
                        warn!("Failed to set read timeout for TRID client: {}", e);
                    }

                    debug!("Successfully connected to TRID");
                    self.trid_client = Some(Arc::new(Mutex::new(stream)));
                }
                Err(e) => {
                    error!("Failed to connect to TRID: {}", e);
                    return Err(e);
                }
            }
        }
        Ok(self.trid_client.as_ref().unwrap().clone())
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
            stream.shutdown(Shutdown::Both).unwrap_or_else(|e| {
                warn!("Failed to shutdown RF256 client stream: {}", e);
            });
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

    fn reconnect_trid_client(&mut self) -> io::Result<()> {
        if let Some(client) = &self.trid_client {
            debug!(
                "Reconnecting to TRID at {}:{}",
                self.config.trid_ip, self.config.trid_port
            );
            let mut stream = client.lock().unwrap();
            stream.shutdown(Shutdown::Both).unwrap_or_else(|e| {
                warn!("Failed to shutdown TRID client stream: {}", e);
            });
            match TcpStream::connect_timeout(
                &SocketAddr::new(self.config.trid_ip.parse().unwrap(), self.config.trid_port),
                Duration::from_secs(1),
            ) {
                Ok(new_stream) => {
                    *stream = new_stream;
                    if let Err(e) =
                        stream.set_read_timeout(Some(std::time::Duration::from_millis(100)))
                    {
                        warn!("Failed to set read timeout for TRID client: {}", e);
                        return Err(e);
                    }
                    debug!("Successfully reconnected to TRID");
                }
                Err(e) => {
                    error!("Failed to reconnect to TRID: {}", e);
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
            stream.shutdown(Shutdown::Both).unwrap_or_else(|e| {
                warn!(
                    "Failed to shutdown Standa client stream at index {}: {}",
                    index, e
                );
            });
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
            let trid_client = self.get_trid_client()?;

            debug!("Creating new SingleAxis controller for index {}", index);
            self.axes[index] = Some(SingleAxis::new(
                rf256_client,
                self.rf256_ids[index],
                trid_client,
                self.trid_ids[index],
                standa_client,
            ));
            debug!(
                "Successfully created SingleAxis controller for index {}",
                index
            );
        }

        Ok(self.axes[index].as_mut().unwrap())
    }

    pub fn get_velocity(&mut self, index: usize) -> io::Result<u32> {
        debug!("Getting velocity for axis {}", index);
        let axis = self.get_axis(index)?;
        let velocity = axis.get_velocity();
        debug!("Got velocity {} for axis {}", velocity, index);
        Ok(velocity)
    }

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

    pub fn get_acceleration(&mut self, index: usize) -> io::Result<u16> {
        debug!("Getting acceleration for axis {}", index);
        let axis = self.get_axis(index)?;
        let acceleration = axis.get_acceleration();
        debug!("Got acceleration {} for axis {}", acceleration, index);
        Ok(acceleration)
    }

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

    pub fn get_deceleration(&mut self, index: usize) -> io::Result<u16> {
        debug!("Getting deceleration for axis {}", index);
        let axis = self.get_axis(index)?;
        let deceleration = axis.get_deceleration();
        debug!("Got deceleration {} for axis {}", deceleration, index);
        Ok(deceleration)
    }

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

    pub fn get_position_window(&mut self, index: usize) -> io::Result<f32> {
        debug!("Getting position window for axis {}", index);
        let axis = self.get_axis(index)?;
        let position_window = axis.get_position_window();
        debug!("Got position window {} for axis {}", position_window, index);
        Ok(position_window)
    }

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

    pub fn get_time_limit(&mut self, index: usize) -> io::Result<Duration> {
        debug!("Getting time limit for axis {}", index);
        let axis = self.get_axis(index)?;
        let time_limit = axis.get_time_limit();
        debug!("Got time limit {} for axis {}", time_limit.as_secs(), index);
        Ok(time_limit)
    }

    pub fn set_time_limit(&mut self, index: usize, time_limit: Duration) -> io::Result<()> {
        debug!(
            "Setting time limit to {} for axis {}",
            time_limit.as_secs(),
            index
        );
        let axis = self.get_axis(index)?;
        axis.set_time_limit(time_limit);
        debug!(
            "Successfully set time limit to {} for axis {}",
            time_limit.as_secs(),
            index
        );
        Ok(())
    }

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

    pub fn position(&mut self, index: usize) -> io::Result<f32> {
        debug!("Getting position for axis {}", index);
        let axis = self.get_axis(index)?;
        let result = axis.position();
        if let Err(ref e) = result {
            error!("Failed to get position for axis {}: {}", index, e);
            if e.kind() == io::ErrorKind::BrokenPipe {
                error!("Detected broken pipe, attempting to reconnect RF256 client");
                self.reconnect_rf256_client()?;
            } else if e.kind() == io::ErrorKind::InvalidData {
                error!(
                    "Detected invalid data, attempting to reconnect RF256 client for index {}",
                    index
                );
                self.reconnect_rf256_client()?;
                error!("Reconnected RF256 client for axis {}", index);
            }
        } else if let Ok(pos) = result {
            debug!("Got position {} for axis {}", pos, index);
        }
        result
    }

    pub fn temperature(&mut self, index: usize) -> io::Result<u16> {
        debug!("Getting temperature for axis {}", index);
        let axis = self.get_axis(index)?;
        let result = axis.temperature();
        if let Err(ref e) = result {
            error!("Failed to get temperature for axis {}: {}", index, e);
            if e.kind() == io::ErrorKind::BrokenPipe || e.kind() == io::ErrorKind::WouldBlock {
                warn!("Detected broken pipe or would block, attempting to reconnect Standa client for index {}", index);
                self.reconnect_trid_client()?;
            }
        } else {
            debug!("Successfully got temperature for axis {}", index);
        }
        result
    }

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

#[derive(Deserialize, Debug, Serialize)]
pub struct MultiAxisConfig {
    pub rf256_ip: String,
    pub rf256_port: u16,

    pub trid_ip: String,
    pub trid_port: u16,

    pub upper_standa_ip: String,
    pub upper_standa_port: u16,
    pub upper_rf256_id: u8,
    pub upper_trid_id: u8,

    pub lower_standa_ip: String,
    pub lower_standa_port: u16,
    pub lower_rf256_id: u8,
    pub lower_trid_id: u8,

    pub left_standa_ip: String,
    pub left_standa_port: u16,
    pub left_rf256_id: u8,
    pub left_trid_id: u8,

    pub right_standa_ip: String,
    pub right_standa_port: u16,
    pub right_rf256_id: u8,
    pub right_trid_id: u8,
}

impl Default for MultiAxisConfig {
    fn default() -> Self {
        Self {
            rf256_ip: "192.168.0.51".to_string(),
            rf256_port: 60003,

            trid_ip: "192.168.0.51".to_string(),
            trid_port: 60002,

            upper_standa_ip: "192.168.0.204".to_string(),
            upper_standa_port: 2000,
            upper_rf256_id: 6,
            upper_trid_id: 0,

            lower_standa_ip: "192.168.0.204".to_string(),
            lower_standa_port: 3000,
            lower_rf256_id: 15,
            lower_trid_id: 1,

            left_standa_ip: "192.168.0.205".to_string(),
            left_standa_port: 2000,
            left_rf256_id: 5,
            left_trid_id: 2,

            right_standa_ip: "192.168.0.205".to_string(),
            right_standa_port: 3000,
            right_rf256_id: 4,
            right_trid_id: 3,
        }
    }
}
