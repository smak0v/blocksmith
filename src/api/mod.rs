mod configuration;
mod errors;
mod handlers;
mod startup;

use anyhow::Result;
use tokio::sync::mpsc::UnboundedSender;

use crate::api::startup::Application;
use crate::domain::transaction::Transaction;

pub async fn launch_and_run_api_module(api_tx_sender: UnboundedSender<Transaction>) -> Result<()> {
    let configuration = configuration::get_configuration()?;
    let application = Application::build(configuration, api_tx_sender).await?;

    Ok(application.run_util_stopped().await?)
}
