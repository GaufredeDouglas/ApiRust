use rocket_db_pools::Database;

#[derive(Database)]
#[database("pokemon_db")]
pub struct PokemonDb(sqlx::PgPool);

pub async fn run_migrations(rocket: rocket::Rocket<rocket::Build>) -> rocket::Rocket<rocket::Build> {
    let db = PokemonDb::fetch(&rocket).expect("database connection");
    let conn = &db.0;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS pokemons (
            id SERIAL PRIMARY KEY,
            data JSONB NOT NULL
        )"
    )
    .execute(conn)
    .await
    .expect("can create table");

    rocket
}
