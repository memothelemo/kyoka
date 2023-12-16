use actix_web::{
    web::{self, ServiceConfig},
    HttpResponse,
};

pub async fn health() -> HttpResponse {
    HttpResponse::Ok().body("I'm healthy and alive!")
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.route("/health", web::get().to(health));
}
