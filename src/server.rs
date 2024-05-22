use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use mime_guess::from_path;
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "web/dist"]
struct Asset;

async fn ws(
    path: web::Path<String>,
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, actix_web::Error> {
    Ok(HttpResponse::NotFound().finish())
}

fn handle_embedded_file(path: &str) -> HttpResponse {
    match Asset::get(path) {
        Some(content) => HttpResponse::Ok()
            .content_type(from_path(path).first_or_octet_stream().as_ref())
            .body(content.data.into_owned()),
        None => HttpResponse::NotFound().body("404 Not Found"),
    }
}

async fn index() -> HttpResponse {
    handle_embedded_file("index.html")
}

async fn dist(path: web::Path<String>) -> HttpResponse {
    handle_embedded_file(&path.as_str())
}
pub async fn run() -> Result<(), anyhow::Error> {
    let server = HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .route("/ws/{mode:(control)}", web::get().to(ws))
            .route("/", web::get().to(index))
            .route("/{_:.*}", web::get().to(dist))
    });

    server.bind(format!("0.0.0.0:{}", 3000))?.run().await?;

    Ok(())
}
