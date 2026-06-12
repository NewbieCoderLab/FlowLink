use flowlink_lib::{
    config::defaults::default_app_config,
    discovery::{start_discovery_tasks, udp::broadcast_destinations},
    identity::DeviceIdentity,
};

#[tokio::main]
async fn main() {
    let seconds = arg_value("--seconds")
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(3);
    let mut config = default_app_config();
    config.network.listen_port = arg_value("--service-port")
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(config.network.listen_port);
    config.discovery.udp_port = arg_value("--udp-port")
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(config.discovery.udp_port);

    let identity = DeviceIdentity::generate();
    println!(
        "local: id={} name={} service_port={} udp_port={}",
        identity.device_id,
        identity.device_name,
        config.network.listen_port,
        config.discovery.udp_port
    );
    println!(
        "udp broadcast targets: {:?}",
        broadcast_destinations(config.discovery.udp_port)
    );

    let (tx, mut rx) = tokio::sync::mpsc::channel(256);
    let _runtime = start_discovery_tasks(
        identity,
        config.network.clone(),
        config.discovery.clone(),
        tx,
    )
    .expect("failed to start discovery tasks");

    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(seconds);
    let mut discovered = Vec::new();
    while tokio::time::Instant::now() < deadline {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        match tokio::time::timeout(remaining, rx.recv()).await {
            Ok(Some(peer)) => {
                println!(
                    "peer: id={} name={} source={:?} addresses={:?}",
                    peer.device_id, peer.device_name, peer.source, peer.addresses
                );
                discovered.push(peer.device_id);
            }
            Ok(None) => break,
            Err(_) => break,
        }
    }

    println!("discovered {} peer(s)", discovered.len());
    if discovered.is_empty() {
        std::process::exit(2);
    }
}

fn arg_value(name: &str) -> Option<String> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == name {
            return args.next();
        }
    }
    None
}
