mod configuration;
mod errors;
mod handlers;
mod startup;

use anyhow::Result;
use libp2p::Swarm;

use std::sync::{Arc, Mutex};

use crate::api::startup::Application;
use crate::app_state::AppState;
use crate::p2p::ChainBehaviour;

pub async fn launch_and_run_api_module(
    swarm: Arc<Mutex<Swarm<ChainBehaviour>>>,
    node_state: Arc<Mutex<AppState>>,
) -> Result<()> {
    let configuration = configuration::get_configuration()?;
    let application = Application::build(configuration, swarm, node_state).await?;

    Ok(application.run_util_stopped().await?)
}
