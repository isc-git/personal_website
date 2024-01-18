use std::{net, path::PathBuf, process};

use argh::FromArgs;
use axum::Router;
use axum_server::tls_rustls::RustlsConfig;
use tower_http::services;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(FromArgs)]
/// ssl requirements
struct Args {
    /// certificate for https
    #[argh(option, short = 'c')]
    cert: PathBuf,

    #[argh(option, short = 'k')]
    /// private key for cert
    key: PathBuf,

    #[argh(option, short = 'p')]
    /// port
    port: u16,

    #[argh(option, short = 'd')]
    /// seconds between rustls config reload
    reload: u64,
}

#[tokio::main]
async fn main() {
    // enable logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                ["website=debug", "tower_http=debug", "axum::rejection=trace"]
                    .join(",")
                    .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // read commandline arguments
    let args: Args = argh::from_env();

    // ensure that the cert and key exists
    if !args.cert.exists() {
        eprintln!("cert file does not exist: '{}'", args.cert.display());
        process::exit(1);
    }
    if !args.key.exists() {
        eprintln!("key file does not exist: '{}'", args.key.display());
        process::exit(1);
    }

    // configure certificate and private key for https
    let config = RustlsConfig::from_pem_file(&args.cert, &args.key)
        .await
        .unwrap();

    tokio::spawn(reload_config(
        config.clone(),
        args.reload,
        args.cert.clone(),
        args.key.clone(),
    ));

    // setup app to serve static files
    let serve_dir = services::ServeDir::new("assets")
        .not_found_service(services::ServeFile::new("assets/index.html"));
    let app = Router::new()
        .fallback_service(serve_dir)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let address = net::SocketAddr::from(([0, 0, 0, 0], args.port));
    println!("listen on {}", address);
    axum_server::bind_rustls(address, config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn reload_config<P: AsRef<std::path::Path>>(
    config: RustlsConfig,
    seconds: u64,
    cert: P,
    key: P,
) {
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(seconds)).await;
        config.reload_from_pem_file(&cert, &key).await.unwrap();
    }
}
