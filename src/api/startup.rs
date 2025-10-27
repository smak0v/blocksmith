use actix_web::{
    App, HttpServer,
    dev::Server,
    web::{self, Data},
};
use libp2p::Swarm;
use tracing::info;
use tracing_actix_web::TracingLogger;

use std::fmt::{Debug, Formatter};
use std::io::Error;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use crate::api::configuration::Settings;
use crate::api::handlers;
use crate::app_state::AppState;
use crate::p2p::ChainBehaviour;

pub struct Application {
    server: Server,
}

pub struct SwarmWrapper(pub Arc<Mutex<Swarm<ChainBehaviour>>>);

#[derive(Debug)]
pub struct NodeStateWrapper(pub Arc<Mutex<AppState>>);

impl Application {
    pub async fn build(
        configuration: Settings,
        swarm: Arc<Mutex<Swarm<ChainBehaviour>>>,
        node_state: Arc<Mutex<AppState>>,
    ) -> Result<Self, Error> {
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(address.clone()).expect(&format!(
            "failed to bind port {}",
            configuration.application.port
        ));
        let server = Application::run(listener, swarm, node_state)?;

        info!("Server started on {}", address);

        Ok(Self { server })
    }

    pub async fn run_util_stopped(self) -> Result<(), Error> {
        self.server.await
    }

    fn run(
        listener: TcpListener,
        swarm: Arc<Mutex<Swarm<ChainBehaviour>>>,
        node_state: Arc<Mutex<AppState>>,
    ) -> Result<Server, Error> {
        let swarm = Data::new(SwarmWrapper(swarm));
        let node_state = Data::new(NodeStateWrapper(node_state));
        let server = HttpServer::new(move || {
            App::new()
                .wrap(TracingLogger::default())
                .route("/transaction", web::post().to(handlers::submit_transaction))
                .app_data(swarm.clone())
                .app_data(node_state.clone())
        })
        .listen(listener)?
        .run();

        Ok(server)
    }
}

impl Debug for SwarmWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SwarmWrapper")
    }
}
