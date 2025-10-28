use actix_web::{
    App, HttpServer,
    dev::Server,
    web::{self, Data},
};
use tokio::sync::mpsc::UnboundedSender;
use tracing::info;
use tracing_actix_web::TracingLogger;

use std::io::Error;
use std::net::TcpListener;

use crate::api::configuration::Settings;
use crate::api::handlers;
use crate::domain::transaction::Transaction;

pub struct Application {
    server: Server,
}

#[derive(Debug)]
pub struct ApiTxSender(pub UnboundedSender<Transaction>);

impl Application {
    pub async fn build(
        configuration: Settings,
        api_tx_sender: UnboundedSender<Transaction>,
    ) -> Result<Self, Error> {
        let address = format!("{}:0", configuration.application.host);
        let listener = TcpListener::bind(address.clone()).expect(&format!(
            "failed to bind port {}",
            configuration.application.port
        ));
        let port = listener.local_addr()?.port();
        let server = Application::run(listener, api_tx_sender)?;

        info!(
            "Server started on {}:{}",
            configuration.application.host, port
        );

        Ok(Self { server })
    }

    pub async fn run_util_stopped(self) -> Result<(), Error> {
        self.server.await
    }

    fn run(
        listener: TcpListener,
        api_tx_sender: UnboundedSender<Transaction>,
    ) -> Result<Server, Error> {
        let api_tx_sender = Data::new(ApiTxSender(api_tx_sender));
        let server = HttpServer::new(move || {
            App::new()
                .wrap(TracingLogger::default())
                .route("/transaction", web::post().to(handlers::submit_transaction))
                .app_data(api_tx_sender.clone())
        })
        .listen(listener)?
        .run();

        Ok(server)
    }
}
