use actix_web::web::{self, ServiceConfig};
use actix_web::HttpResponse;
use serde::Deserialize;

use super::AppContext;

#[tracing::instrument]
pub async fn health() -> HttpResponse {
    HttpResponse::Ok().body("I'm healthy and alive!")
}

#[derive(Debug, Deserialize)]
struct QueryParams {
    pub shard: Option<u64>,
}

#[tracing::instrument(skip_all, fields(params.id = ?query.shard))]
pub async fn index(
    query: web::Query<QueryParams>,
    ctx: web::Data<AppContext>,
) -> HttpResponse {
    let shard = query.shard;
    if shard.is_none() && ctx.big_queue {
        tracing::warn!(
            "No shard id set, defaulting to 0. Will not bucket requests correctly!"
        );
    }

    let shard = shard.unwrap_or(0);
    ctx.queue.request([shard, 1]).await;

    HttpResponse::Ok().body("You're good to initialize session. :)")
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.route("/health", web::get().to(health))
        .route("/queue", web::post().to(index));
}
