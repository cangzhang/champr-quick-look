use axum::{extract::Extension, routing::get, Router};
use diesel_async::{
    pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager},
    AsyncPgConnection,
};
use dotenvy::dotenv;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;

use std::{env, net::SocketAddr, time::Duration};

pub mod api;
pub mod config;
pub mod errors;

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<AsyncPgConnection>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    // initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(db_url);
    let pool = Pool::builder(config).build().unwrap();

    let agent: ureq::Agent = ureq::AgentBuilder::new()
        .timeout_read(Duration::from_secs(10))
        .timeout_write(Duration::from_secs(10))
        .build();

    let api_routes = Router::new()
        .route("/sources", get(api::list_sources))
        .route(
            "/source/:source/builds/:champion",
            get(api::get_lastest_build),
        )
        .route(
            "/source/:source/runes/:champion",
            get(api::get_lastest_build),
        )
        .route("/data-dragon/champions", get(api::list_champion_map))
        .route("/data-dragon/runes", get(api::list_runes_reforged))
        .layer(Extension(agent));

    let app = Router::new().nest("/api", api_routes).layer(
        TraceLayer::new_for_http()
            .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
            .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
    );

    let addr = SocketAddr::from(([0, 0, 0, 0], 3030));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
