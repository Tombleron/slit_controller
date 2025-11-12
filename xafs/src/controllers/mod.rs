use std::{net::SocketAddr, time::Duration};

use em2rs::Em2rs;
use lir::LIR;
use trid::Trid;
use utilities::{command_executor::CommandExecutor, lazy_tcp::LazyTcpStream};

use crate::{
    command_executor::{
        motor::{Em2rsHandler, command_sender::Em2rsCommandSender},
        sensors::{SensorsHandler, command_sender::SensorsCommandSender},
    },
    controllers::{
        attenuator::controller::AttenuatorController, collimator::controller::CollimatorController,
        config::XafsConfig, cooled_slit::controller::CooledSlitController,
        water_input::controller::WaterInputController,
    },
};

pub mod attenuator;
pub mod collimator;
pub mod config;
pub mod cooled_slit;
pub mod water_input;

const READ_TIMEOUT: Duration = Duration::from_millis(100);
const WRITE_TIMEOUT: Duration = Duration::from_millis(100);
const CONNECT_TIMEOUT: Duration = Duration::from_millis(100);
const MAX_RETRIES: u32 = 3;

pub fn create_sensors(
    config: &XafsConfig,
) -> (CommandExecutor<SensorsHandler>, SensorsCommandSender) {
    let sensors_scoket_addr =
        SocketAddr::new(config.sensors_ip.parse().unwrap(), config.sensors_port);

    let sensors_tcp_stream = LazyTcpStream::new(
        sensors_scoket_addr,
        MAX_RETRIES,
        READ_TIMEOUT,
        WRITE_TIMEOUT,
        CONNECT_TIMEOUT,
    );

    let sensors_handler = SensorsHandler::new(
        sensors_tcp_stream,
        vec![
            LIR::new(
                config.slit_controller.upper_axis.lir_id,
                config.slit_controller.upper_axis.lir_step,
            ),
            LIR::new(
                config.slit_controller.lower_axis.lir_id,
                config.slit_controller.lower_axis.lir_step,
            ),
            LIR::new(
                config.slit_controller.right_axis.lir_id,
                config.slit_controller.right_axis.lir_step,
            ),
            LIR::new(
                config.slit_controller.left_axis.lir_id,
                config.slit_controller.left_axis.lir_step,
            ),
            LIR::new(
                config.attenuator.axis.lir_id,
                config.attenuator.axis.lir_step,
            ),
        ],
        vec![
            // Knifes temperature
            Trid::new(
                config.slit_controller.knife_trid_id,
                config.slit_controller.upper_axis.knife_trid_axis,
            ),
            Trid::new(
                config.slit_controller.knife_trid_id,
                config.slit_controller.lower_axis.knife_trid_axis,
            ),
            Trid::new(
                config.slit_controller.knife_trid_id,
                config.slit_controller.right_axis.knife_trid_axis,
            ),
            Trid::new(
                config.slit_controller.knife_trid_id,
                config.slit_controller.left_axis.knife_trid_axis,
            ),
            // Water temperature
            Trid::new(
                config.slit_controller.water_trid_id,
                config.slit_controller.upper_axis.water_trid_axis,
            ),
            Trid::new(
                config.slit_controller.water_trid_id,
                config.slit_controller.lower_axis.water_trid_axis,
            ),
            Trid::new(
                config.slit_controller.water_trid_id,
                config.slit_controller.right_axis.water_trid_axis,
            ),
            Trid::new(
                config.slit_controller.water_trid_id,
                config.slit_controller.left_axis.water_trid_axis,
            ),
            // Water input temperature
            Trid::new(
                config.water_input.trid_id,
                config.water_input.axis.trid_axis,
            ),
            // Collimator temperature
            Trid::new(
                config.collimator.trid_id,
                config.collimator.input_axis.trid_axis,
            ),
            Trid::new(
                config.collimator.trid_id,
                config.collimator.output_axis.trid_axis,
            ),
        ],
    );

    let sensors_command_executor = CommandExecutor::new(sensors_handler);
    let sensors_command_sender = SensorsCommandSender::new(sensors_command_executor.sender());

    (sensors_command_executor, sensors_command_sender)
}

pub fn create_em2rs(config: &XafsConfig) -> (CommandExecutor<Em2rsHandler>, Em2rsCommandSender) {
    let em2rs_socket_addr = SocketAddr::new(config.em2rs_ip.parse().unwrap(), config.em2rs_port);
    let em2rs_tcp_stream = LazyTcpStream::new(
        em2rs_socket_addr,
        MAX_RETRIES,
        READ_TIMEOUT,
        WRITE_TIMEOUT,
        CONNECT_TIMEOUT,
    );

    let em2rs_handler = Em2rsHandler::new(
        em2rs_tcp_stream,
        [
            // Slit
            Em2rs::new(
                config.slit_controller.upper_axis.em2rs_id,
                config.slit_controller.upper_axis.em2rs_low_limit,
                config.slit_controller.upper_axis.em2rs_high_limit,
            ),
            Em2rs::new(
                config.slit_controller.lower_axis.em2rs_id,
                config.slit_controller.lower_axis.em2rs_low_limit,
                config.slit_controller.lower_axis.em2rs_high_limit,
            ),
            Em2rs::new(
                config.slit_controller.right_axis.em2rs_id,
                config.slit_controller.right_axis.em2rs_low_limit,
                config.slit_controller.right_axis.em2rs_high_limit,
            ),
            Em2rs::new(
                config.slit_controller.left_axis.em2rs_id,
                config.slit_controller.left_axis.em2rs_low_limit,
                config.slit_controller.left_axis.em2rs_high_limit,
            ),
            // Attenuator
            Em2rs::new(
                config.attenuator.axis.em2rs_id,
                config.attenuator.axis.em2rs_low_limit,
                config.attenuator.axis.em2rs_high_limit,
            ),
        ],
    );

    let em2rs_command_executor = CommandExecutor::new(em2rs_handler);
    let em2rs_command_sender = Em2rsCommandSender::new(em2rs_command_executor.sender());

    (em2rs_command_executor, em2rs_command_sender)
}

pub fn create_controllers(
    config: &XafsConfig,
) -> (
    CollimatorController,
    CooledSlitController,
    AttenuatorController,
    WaterInputController,
    CommandExecutor<Em2rsHandler>,
    CommandExecutor<SensorsHandler>,
) {
    let (em2rs_command_executor, em2rs_command_sender) = create_em2rs(config);
    let (sensors_command_executor, sensors_command_sender) = create_sensors(config);

    let collimator_controller = collimator::create_controller(sensors_command_sender.clone());
    let slit_controller = cooled_slit::create_controller(
        &config.slit_controller,
        em2rs_command_sender.clone(),
        sensors_command_sender.clone(),
    );
    let attenuator_controller = attenuator::create_controller(
        &config.attenuator,
        em2rs_command_sender.clone(),
        sensors_command_sender.clone(),
    );
    let water_input_controller = water_input::create_controller(sensors_command_sender.clone());

    (
        collimator_controller,
        slit_controller,
        attenuator_controller,
        water_input_controller,
        em2rs_command_executor,
        sensors_command_executor,
    )
}
