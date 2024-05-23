use actix_web::{error, web, App, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use mime_guess::from_path;
use rust_embed::Embed;
use tracing::{error, info};

use crate::controller::Controller;

#[derive(Embed)]
#[folder = "web-ui/dist"]
struct Asset;

async fn ws(
    path: web::Path<String>,
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, actix_web::Error> {
    match path.as_str() {
        "control" => {
            let controller = Controller::new(&req.connection_info()).map_err(|err| {
                error!("Failed to create controller: {}", err);
                error::ErrorInternalServerError(err)
            })?;

            ws::start(controller, &req, stream)
        }
        _ => Ok(HttpResponse::NotFound().finish()),
    }
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
            .route("/api/{mode:(control)}", web::get().to(ws))
            .route("/", web::get().to(index))
            .route("/{_:.*}", web::get().to(dist))
    });

    info!("Starting webserver");
    server.bind(format!("0.0.0.0:{}", 3000))?.run().await?;

    Ok(())
}
