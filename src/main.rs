#[macro_use]
extern crate diesel;

pub mod models;
pub mod schema;

use dotenv::dotenv;
use std::env;
use tera::{Context, Tera};

use diesel::pg::PgConnection;
use diesel::prelude::*;

use diesel::r2d2::{self, ConnectionManager};
use diesel::r2d2::{Pool, PooledConnection};

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

use self::models::{NewPost, NewPostHandler, Post};
use self::schema::posts;
use self::schema::posts::dsl::*;

#[get("/")]
async fn index(pool: web::Data<DbPool>, template_manager: web::Data<tera::Tera>) -> impl Responder {
    let mut conn: PooledConnection<ConnectionManager<PgConnection>> =
        pool.get().expect("Problemas al traer la base de datos");

    match web::block(move || posts.load::<Post>(&mut conn)).await {
        Ok(data) => {
            let data = data.unwrap();
            let mut ctx: Context = tera::Context::new();
            ctx.insert("posts", &data);
            HttpResponse::Ok()
                .content_type("text/html")
                .body(template_manager.render("index.html", &ctx).unwrap())
        }
        Err(err) => HttpResponse::Ok().body("Error al recibir la data"),
    }
}

#[get("/blog/{blog_slug}")]
async fn get_post(
    pool: web::Data<DbPool>,
    template_manager: web::Data<tera::Tera>,
    blog_slug: web::Path<String>,
) -> impl Responder {
    let mut conn: PooledConnection<ConnectionManager<PgConnection>> =
        pool.get().expect("Problemas al traer la base de datos");

    let url_slug: String = blog_slug.into_inner();
    match web::block(move || posts.filter(slug.eq(url_slug)).load::<Post>(&mut conn)).await {
        Ok(data) => {
            let data = data.unwrap();
            if data.len() == 0 {
                return HttpResponse::NotFound().finish();
            }

            let mut ctx: Context = tera::Context::new();
            ctx.insert("posts", &data);
            HttpResponse::Ok()
                .content_type("text/html")
                .body(template_manager.render("posts.html", &ctx).unwrap())
        }
        Err(err) => HttpResponse::Ok().body("Error al recibir la data"),
    }
}

#[post("/new_post")]
async fn new_post(pool: web::Data<DbPool>, item: web::Json<NewPostHandler>) -> impl Responder {
    let conn = pool.get().expect("Problemas al traer la base de datos");

    match web::block(move || Post::create_post(conn, &item)).await {
        Ok(data) => {
            return HttpResponse::Ok().body(format!("{:?}", data));
        }
        Err(err) => HttpResponse::Ok().body("Error al recibir la data"),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("db url variable no encontrada");

    let connection = ConnectionManager::<PgConnection>::new(db_url);

    let pool = Pool::builder()
        .build(connection)
        .expect("No se pudo construir la Pool");

    HttpServer::new(move || {
        let tera = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();
        App::new()
            .service(index)
            .service(new_post)
            .service(get_post)
            .data(pool.clone())
            .data(tera)
    })
    .bind(("0.0.0.0", 9900))
    .unwrap()
    .run()
    .await
}
