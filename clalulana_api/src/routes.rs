use actix_web::web;

use crate::handlers::{auth, health, users};

/// Configure all API routes under `/api/v1`.
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            // Health check
            .service(web::resource("/health").route(web::get().to(health::health_check)))

            // Auth routes (public)
            .service(
                web::scope("/auth")
                    .route("/register", web::post().to(auth::register))
                    .route("/login", web::post().to(auth::login)),
            )

            // User routes (authenticated)
            .service(
                web::scope("/users")
                    .route("", web::get().to(users::get_all_users))
                    .route("/me", web::get().to(users::get_current_user))
                    .route("/{id}", web::get().to(users::get_user_by_id))
                    .route("/{id}", web::put().to(users::update_user))
                    .route("/{id}", web::delete().to(users::delete_user)),
            ),
    );
}
