use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{
    communication::{Command, CommandEnvelope, CommandError},
    models::SharedState,
};

async fn handle_get_command(
    envelope: CommandEnvelope,
    shared_state: &Arc<Mutex<SharedState>>,
) -> Result<(), CommandError> {
    let CommandEnvelope {
        command: Command::Get { axis, property },
        sender: _,
    } = envelope
    else {
        unreachable!("Only GET commands are supported")
    };

    let mut shared_state = shared_state.lock().await;

    todo!()
}
