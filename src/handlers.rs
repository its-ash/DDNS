use actix_web::{web, HttpResponse, HttpRequest};
use actix_session::Session;
use sqlx::SqlitePool;
use tera::Tera;
use base64::{Engine as _, engine::general_purpose};
use rand::Rng;
use serde::Deserialize;

use crate::models::{CreateHostRequest, AddHostnameRequest, LoginRequest, UpdateIpRequest, AdminPassword, BaseDomain};
use crate::db;

#[derive(Deserialize)]
pub struct ErrorQuery {
    error: Option<String>,
}

pub async fn index(
    tmpl: web::Data<Tera>, 
    session: Session,
    query: web::Query<ErrorQuery>,
) -> HttpResponse {
    let is_logged_in = session.get::<bool>("logged_in").unwrap_or(Some(false)).unwrap_or(false);
    
    if is_logged_in {
        HttpResponse::Found()
            .append_header(("Location", "/dashboard"))
            .finish()
    } else {
        let mut ctx = tera::Context::new();
        if query.error.is_some() {
            ctx.insert("error", &true);
        }
        let rendered = tmpl.render("login.html", &ctx).unwrap_or_else(|_| "Error".to_string());
        HttpResponse::Ok().content_type("text/html").body(rendered)
    }
}

pub async fn login(
    form: web::Form<LoginRequest>,
    session: Session,
    admin_password: web::Data<AdminPassword>,
) -> HttpResponse {
    if form.password == admin_password.0 {
        session.insert("logged_in", true).unwrap();
        HttpResponse::Found()
            .append_header(("Location", "/dashboard"))
            .finish()
    } else {
        HttpResponse::Found()
            .append_header(("Location", "/?error=invalid"))
            .finish()
    }
}

pub async fn logout(session: Session) -> HttpResponse {
    session.purge();
    HttpResponse::Found()
        .append_header(("Location", "/"))
        .finish()
}

pub async fn dashboard(
    tmpl: web::Data<Tera>,
    session: Session,
    pool: web::Data<SqlitePool>,
    base_domain: web::Data<BaseDomain>,
) -> HttpResponse {
    let is_logged_in = session.get::<bool>("logged_in").unwrap_or(Some(false)).unwrap_or(false);
    
    if !is_logged_in {
        return HttpResponse::Found()
            .append_header(("Location", "/"))
            .finish();
    }

    let hosts = db::get_all_hosts_with_config(&pool).await.unwrap_or_default();
    let configs = db::get_all_configs(&pool).await.unwrap_or_default();
    
    let mut ctx = tera::Context::new();
    ctx.insert("hosts", &hosts);
    ctx.insert("configs", &configs);
    ctx.insert("base_domain", &base_domain.0);
    
    let rendered = tmpl.render("dashboard.html", &ctx).unwrap_or_else(|_| "Error".to_string());
    HttpResponse::Ok().content_type("text/html").body(rendered)
}

fn generate_credentials() -> (String, String) {
    let mut rng = rand::thread_rng();
    let username: String = (0..12)
        .map(|_| {
            let idx = rng.gen_range(0..36);
            if idx < 26 {
                (b'a' + idx) as char
            } else {
                (b'0' + (idx - 26)) as char
            }
        })
        .collect();
    
    let password: String = (0..16)
        .map(|_| {
            let idx = rng.gen_range(0..62);
            if idx < 26 {
                (b'a' + idx) as char
            } else if idx < 52 {
                (b'A' + (idx - 26)) as char
            } else {
                (b'0' + (idx - 52)) as char
            }
        })
        .collect();
    
    (username, password)
}

pub async fn create_host(
    form: web::Form<CreateHostRequest>,
    session: Session,
    pool: web::Data<SqlitePool>,
    base_domain: web::Data<BaseDomain>,
) -> HttpResponse {
    let is_logged_in = session.get::<bool>("logged_in").unwrap_or(Some(false)).unwrap_or(false);
    
    if !is_logged_in {
        return HttpResponse::Unauthorized().finish();
    }

    let hostname = format!("{}.{}", form.subdomain, base_domain.0);

    let config_id = if let Some(cid) = form.config_id {
        // Add to existing config
        cid
    } else {
        // Create new config with new credentials
        let (username, password) = generate_credentials();
        match db::create_config(&pool, &username, &password).await {
            Ok(id) => id,
            Err(_) => return HttpResponse::Found()
                .append_header(("Location", "/dashboard?error=config_create_failed"))
                .finish(),
        }
    };

    match db::create_host(&pool, &hostname, config_id).await {
        Ok(_) => HttpResponse::Found()
            .append_header(("Location", "/dashboard"))
            .finish(),
        Err(_) => HttpResponse::Found()
            .append_header(("Location", "/dashboard?error=exists"))
            .finish(),
    }
}

pub async fn delete_host_handler(
    path: web::Path<i64>,
    session: Session,
    pool: web::Data<SqlitePool>,
) -> HttpResponse {
    let is_logged_in = session.get::<bool>("logged_in").unwrap_or(Some(false)).unwrap_or(false);
    
    if !is_logged_in {
        return HttpResponse::Unauthorized().finish();
    }

    let id = path.into_inner();
    db::delete_host(&pool, id).await.ok();
    
    HttpResponse::Found()
        .append_header(("Location", "/dashboard"))
        .finish()
}

pub async fn update_ip(
    req: HttpRequest,
    query: web::Query<UpdateIpRequest>,
    pool: web::Data<SqlitePool>,
) -> HttpResponse {
    // Extract Basic Auth
    let auth_header = match req.headers().get("Authorization") {
        Some(h) => h.to_str().unwrap_or(""),
        None => return HttpResponse::Unauthorized().body("noauth"),
    };

    if !auth_header.starts_with("Basic ") {
        return HttpResponse::Unauthorized().body("noauth");
    }

    let encoded = &auth_header[6..];
    let decoded = match general_purpose::STANDARD.decode(encoded) {
        Ok(d) => d,
        Err(_) => return HttpResponse::Unauthorized().body("badauth"),
    };

    let credentials = match String::from_utf8(decoded) {
        Ok(c) => c,
        Err(_) => return HttpResponse::Unauthorized().body("badauth"),
    };

    let parts: Vec<&str> = credentials.splitn(2, ':').collect();
    if parts.len() != 2 {
        return HttpResponse::Unauthorized().body("badauth");
    }

    let (username, password) = (parts[0], parts[1]);

    // Verify credentials
    let config = match db::get_config_by_username(&pool, username).await {
        Ok(Some(c)) => c,
        _ => return HttpResponse::Unauthorized().body("badauth"),
    };

    if config.password != password {
        return HttpResponse::Unauthorized().body("badauth");
    }

    // Get IP address
    let ip = if let Some(ip) = &query.myip {
        ip.clone()
    } else {
        // Try to get real IP from proxy headers first
        if let Some(forwarded_for) = req.headers().get("X-Forwarded-For") {
            if let Ok(ff_str) = forwarded_for.to_str() {
                // X-Forwarded-For can be "client, proxy1, proxy2"
                // Take the first (leftmost) IP which is the original client
                ff_str.split(',').next().unwrap_or("").trim().to_string()
            } else {
                // Fallback to peer address
                match req.peer_addr() {
                    Some(addr) => addr.ip().to_string(),
                    None => return HttpResponse::BadRequest().body("noip"),
                }
            }
        } else if let Some(real_ip) = req.headers().get("X-Real-IP") {
            if let Ok(ip_str) = real_ip.to_str() {
                ip_str.to_string()
            } else {
                match req.peer_addr() {
                    Some(addr) => addr.ip().to_string(),
                    None => return HttpResponse::BadRequest().body("noip"),
                }
            }
        } else {
            // No proxy headers, use peer address
            match req.peer_addr() {
                Some(addr) => addr.ip().to_string(),
                None => return HttpResponse::BadRequest().body("noip"),
            }
        }
    };

    // Update IP (this updates the config, which affects all hostnames under it)
    match db::update_config_ip(&pool, username, &ip).await {
        Ok(_) => HttpResponse::Ok().body("good"),
        Err(_) => HttpResponse::InternalServerError().body("dnserr"),
    }
}

pub async fn redirect_to_host(
    req: HttpRequest,
    pool: web::Data<SqlitePool>,
) -> HttpResponse {
    let host_header = match req.headers().get("Host") {
        Some(h) => h.to_str().unwrap_or(""),
        None => return HttpResponse::BadRequest().body("No host header"),
    };

    // Remove port if present
    let hostname = host_header.split(':').next().unwrap_or(host_header);

    match db::get_host_with_config_by_hostname(&pool, hostname).await {
        Ok(Some(host)) => {
            if let Some(ip) = host.current_ip {
                let redirect_url = format!("http://{}", ip);
                HttpResponse::Found()
                    .append_header(("Location", redirect_url))
                    .finish()
            } else {
                HttpResponse::NotFound().body("Host has no IP address registered")
            }
        }
        _ => HttpResponse::NotFound().body("Host not found"),
    }
}

pub async fn add_hostname_to_config(
    path: web::Path<i64>,
    form: web::Form<AddHostnameRequest>,
    session: Session,
    pool: web::Data<SqlitePool>,
    base_domain: web::Data<BaseDomain>,
) -> HttpResponse {
    let is_logged_in = session.get::<bool>("logged_in").unwrap_or(Some(false)).unwrap_or(false);
    
    if !is_logged_in {
        return HttpResponse::Unauthorized().finish();
    }

    let config_id = path.into_inner();
    let hostname = format!("{}.{}", form.subdomain, base_domain.0);

    match db::create_host(&pool, &hostname, config_id).await {
        Ok(_) => HttpResponse::Found()
            .append_header(("Location", "/dashboard"))
            .finish(),
        Err(_) => HttpResponse::Found()
            .append_header(("Location", "/dashboard?error=exists"))
            .finish(),
    }
}
