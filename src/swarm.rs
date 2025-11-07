use libp2p::{
    Swarm, SwarmBuilder, noise::Config as NoiseConfig, tcp::Config as TcpConfig,
    tls::Config as TlsConfig, yamux::Config as YamuxConfig,
};

use crate::p2p::{ChainBehaviour, KEYS};

pub fn init() -> Swarm<ChainBehaviour> {
    let mut swarm = SwarmBuilder::with_existing_identity(KEYS.clone())
        .with_tokio()
        .with_tcp(
            TcpConfig::default(),
            (TlsConfig::new, NoiseConfig::new),
            YamuxConfig::default,
        )
        .expect("failed to setup TCP swarm")
        .with_behaviour(|_| ChainBehaviour::new())
        .expect("failed to setup node P2P behaviour")
        .with_swarm_config(|cfg| cfg)
        .build();

    Swarm::listen_on(
        &mut swarm,
        "/ip4/0.0.0.0/tcp/0"
            .parse()
            .expect("failed to parse node socket"),
    )
    .expect("failed to start swarm");

    swarm
}
