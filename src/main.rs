use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::sync::Mutex;

use actix_files as fs;
use actix_web::{get, web, App, HttpServer, Responder, Result};
use haiku_generator::HaikuGenerator;
use serde::Serialize;

static UNIX_DICTIONARY: &'static str = "/usr/share/dict/words";

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
struct AppState {
    haiku_generator: Mutex<HaikuGenerator>,
}

#[derive(Serialize)]
struct GenerateHaikuResponse {
    haiku: Vec<String>,
}

#[get("/generate_haiku/")]
async fn generate_haiku(data: web::Data<AppState>) -> Result<impl Responder> {
    let hg = data.haiku_generator.lock().unwrap();
    let resp = hg.generate_haiku();
    Ok(web::Json(GenerateHaikuResponse { haiku: resp }))
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    let lines = read_lines(UNIX_DICTIONARY).unwrap();
    let haiku_generator = HaikuGenerator::new(lines.map(|s| s.unwrap().to_lowercase()).collect());
    let app_state = web::Data::new(AppState {
        haiku_generator: Mutex::new(haiku_generator),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(generate_haiku)
            .service(fs::Files::new("/", "./static").show_files_listing())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
