use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{web::Data, App, HttpServer};
use anyhow::{Context, Result};
use bitcoincore_rpc::{Auth, Client};
use config::CONFIG;
use tokio::sync::Mutex;
use tracing::info;

mod app;
mod config;
mod logger;
mod macros;

struct AppState {
    rpc_client: Arc<Mutex<Client>>,
}

#[actix_web::main]
async fn main() -> Result<()> {
    let _guard = logger::initialize(&logger::Scope::Local, &CONFIG.app.log_level)?;

    let rpc_url = "http://localhost:18443";
    let rpc_auth = Auth::UserPass("user".into(), "password".into());
    let client = Arc::new(Mutex::new(Client::new(rpc_url, rpc_auth)?));

    let cookie_jar_key = actix_web::cookie::Key::generate();

    info!("Starting HTTP server at {}", CONFIG.app.server_url());
    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .wrap(
                actix_session::SessionMiddleware::builder(
                    actix_session::storage::CookieSessionStore::default(),
                    cookie_jar_key.clone(),
                )
                .cookie_content_security(actix_session::config::CookieContentSecurity::Private)
                .build(),
            )
            .app_data(Data::new(AppState {
                rpc_client: client.clone(),
            }))
            .service(app::services())
    })
    .bind(CONFIG.app.server_url())
    .context("could not bind server")?
    .run()
    .await
    .context("could not run server")?;

    Ok(())
}
