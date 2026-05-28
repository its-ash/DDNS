mod models;
mod db;
mod handlers;
mod dns_server;

use actix_web::{web, App, HttpServer, middleware};
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::cookie::Key;
use actix_files as fs;
use tera::Tera;
use std::env;
use crate::models::{AdminPassword, BaseDomain};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let admin_password = env::var("ADMIN_PASSWORD").expect("ADMIN_PASSWORD must be set");
    let base_domain = env::var("BASE_DOMAIN").expect("BASE_DOMAIN must be set");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let session_secret = env::var("SESSION_SECRET").expect("SESSION_SECRET must be set");

    if session_secret.len() < 64 {
        panic!("SESSION_SECRET must be at least 64 characters");
    }

    let pool = db::init_db(&database_url).await.expect("Failed to initialize database");
    
    let tera = Tera::new("templates/**/*.html").expect("Failed to initialize templates");

    let secret_key = Key::from(session_secret.as_bytes());

    println!("Starting DDNS server on {}:{}", host, port);
    println!("Admin dashboard: http://{}:{}", host, port);
    
    // Start DNS server in background
    let dns_pool = pool.clone();
    let dns_domain = base_domain.clone();
    tokio::spawn(async move {
        if let Err(e) = dns_server::start_dns_server(dns_pool, dns_domain).await {
            eprintln!("DNS server error: {}", e);
        }
    });
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(tera.clone()))
            .app_data(web::Data::new(AdminPassword(admin_password.clone())))
            .app_data(web::Data::new(BaseDomain(base_domain.clone())))
            .wrap(middleware::Logger::default())
            .wrap(
                SessionMiddleware::builder(
                    CookieSessionStore::default(),
                    secret_key.clone()
                )
                .cookie_secure(false)
                .cookie_http_only(true)
                .cookie_same_site(actix_web::cookie::SameSite::Lax)
                .build()
            )
            .route("/", web::get().to(handlers::index))
            .route("/login", web::post().to(handlers::login))
            .route("/logout", web::get().to(handlers::logout))
            .route("/dashboard", web::get().to(handlers::dashboard))
            .route("/host/create", web::post().to(handlers::create_host))
            .route("/host/delete/{id}", web::post().to(handlers::delete_host_handler))
            .route("/config/{id}/add-hostname", web::post().to(handlers::add_hostname_to_config))
            .route("/nic/update", web::get().to(handlers::update_ip))
            .route("/update", web::get().to(handlers::update_ip))  // Alternative endpoint for routers
            .route("/redirect", web::get().to(handlers::redirect_to_host))
            .service(fs::Files::new("/static", "templates").show_files_listing())
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}
