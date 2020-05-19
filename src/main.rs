#[macro_use]
extern crate diesel;
extern crate dotenv;
use actix_files::NamedFile;
use actix_web::{get, middleware, post, web, App, Error, HttpResponse, HttpServer};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use dotenv::dotenv;
use std::env;
use std::path::PathBuf;
use yarte::TemplateTrait;
mod actions;
mod models;
mod schema;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[get("/{id}")]
async fn greet(pool: web::Data<DbPool>, id: web::Path<i32>) -> Result<HttpResponse, Error> {
    let page_id = id.into_inner();
    let conn = pool.get().expect("couldn't get db connection from pool");

    let requested_page = web::block(move || actions::find_page_by_id(page_id, &conn))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;

    if let Some(page) = requested_page {
        // The page exists, now let's try to render it
        match page.call() {
            Ok(page) => Ok(HttpResponse::Ok().body(page)),
            Err(_) => {
                Ok(HttpResponse::NotFound().body(format!("Impossible to render page {}", page_id)))
            }
        }
    // Ok(HttpResponse::Ok().json(page))
    } else {
        let res = HttpResponse::NotFound().body(format!("No page found with id: {}", page_id));
        Ok(res)
    }
}

#[get("/")]
async fn index() -> actix_web::Result<NamedFile> {
    let path = PathBuf::from(r"static/index.html");
    Ok(NamedFile::open(path)?)
}

#[post("/create")]
async fn create(
    pool: web::Data<DbPool>,
    form: web::Form<models::NewPage>,
) -> Result<HttpResponse, Error> {
    let conn = pool.get().expect("couldn't get db connection from pool");
    // use web::block to offload blocking Diesel code without blocking server thread
    let new_page = form.into_inner();
    let page = web::block(move || actions::insert_new_page(new_page, &conn))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;

    Ok(HttpResponse::Ok().json(page))
}

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info,diesel=debug");
    env_logger::init();
    dotenv::dotenv().ok();

    // set up database connection pool
    let connspec = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let manager = ConnectionManager::<PgConnection>::new(connspec);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");
    let bind = "127.0.0.1:8080";

    println!("Starting server at: {}", &bind);

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            // set up DB pool to be used with web::Data<Pool> extractor
            .data(pool.clone())
            .wrap(middleware::Logger::default())
            .service(greet)
            .service(index)
            .service(create)
    })
    .bind(&bind)?
    .run()
    .await
}
