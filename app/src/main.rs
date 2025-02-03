use rocket::fairing::AdHoc;
use crate::db::run_migrations;

#[macro_use] extern crate rocket;

mod models;
mod routes;
mod db;

use rocket_db_pools::Database;

#[launch]
async fn rocket() -> _ {
    rocket::build()
        .attach(db::PokemonDb::init())
        .mount("/api", routes![
            routes::get_pokemons,
            routes::get_pokemon,
            routes::create_pokemon,
            routes::update_pokemon,
            routes::delete_pokemon
        ])
        .attach(AdHoc::on_ignite("Database Migrations", run_migrations))
}
