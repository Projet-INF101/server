#[macro_use]
extern crate diesel;

use actix_web::{middleware, web, App, Error, HttpResponse, HttpServer};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use dotenv;
use futures::Future;
use serde::{Serialize, Deserialize, ser::Serializer};

#[macro_use]
mod schema;

use crate::schema::scores;

type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[derive(Debug, Serialize, Queryable)]
struct Score {
    id: i32,
    player: String,
    n_turn: i32,
    median_time: i32,
    disks: i32,
    #[serde(serialize_with = "ser_date")]
    creation_date: chrono::NaiveDateTime,
}

#[derive(Deserialize, Insertable)]
#[table_name = "scores"]
struct NewScore {
    player: String,
    n_turn: i32,
    disks: i32,
    median_time: i32,
}

fn ser_date<'de, S>(date: &chrono::NaiveDateTime, ser: S) -> Result<S::Ok, S::Error> where S: Serializer {
    ser.serialize_str(&format!("{}", date))
}

impl Score {
    fn insert(
        new: NewScore,
        pool: web::Data<Pool>,
    ) -> Result<Score, diesel::result::Error> {
        let conn: &PgConnection = &pool.get().unwrap();

        diesel::insert_into(scores::table).values(&new).get_result(conn)
    }

    fn list(
        pool: web::Data<Pool>,
    ) -> Result<Vec<Score>, diesel::result::Error> {
        let conn: &PgConnection = &pool.get().unwrap();

        scores::table.limit(20).get_results(conn)
    }
}

fn get_scores(
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    // run diesel blocking code
    web::block(move || Score::list(pool)).then(|res| match res {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(e) => Ok(HttpResponse::InternalServerError().body(format!("{:#?}", e)).into()),
    })
}

fn add_score(
    item: web::Json<NewScore>,
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    // run diesel blocking code
    web::block(move || Score::insert(item.into_inner(), pool)).then(|res| match res {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(e) => Ok(HttpResponse::InternalServerError().body(format!("{:#?}", e)).into()),
    })
}

pub fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    dotenv::dotenv().ok();

    let connspec = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let manager = ConnectionManager::<PgConnection>::new(connspec);
    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    println!("Started http server: http://127.0.0.1:7878");

    // Start http server
    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .wrap(middleware::Logger::default())
            .service(
                web::resource("/hanoi/api/v1/scores")
                    .route(web::get().to_async(get_scores))
                    .route(web::post().to_async(add_score)),
            )
    })
    .bind("127.0.0.1:7878")?
    .run()
}
