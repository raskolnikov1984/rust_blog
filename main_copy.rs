#[macro_use]
extern crate diesel;

/* Permite Cargar Variables de Entorno de un archivo .env*/
use dotenv::dotenv;
use std::env;

/* Importacion de Metodos Publicos */
pub mod models;
pub mod schema;

/* Conexion a Postgres*/
use diesel::pg::PgConnection;
use diesel::prelude::*;

fn main() {
    use self::models::{NewPost, Post, PostSimplificado};
    use self::schema::posts;
    use self::schema::posts::dsl::*;

    let new_post = NewPost {
        title: "My Firts Blogpost",
        body: "Lorem impsiingsdd",
        slug: "primer-post",
    };
    dotenv().ok();
    let db_url: String = env::var("DATABASE_URL").expect("DB URL NOT FOUND");
    let mut conn: PgConnection = PgConnection::establish(&db_url).expect("CONECTION ERROR");

    let post: Post = diesel::insert_into(posts::table)
        .values(new_post)
        .get_result(&mut conn)
        .expect("Insertion Failed");

    // Select * from posts limit 1
    let posts_result = posts
        .load::<Post>(&mut conn)
        .expect("ERROR WHEN TRYING TO QUERY");

    for post in posts_result {
        println!("{:?}", post);
    }
}
