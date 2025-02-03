use rocket_db_pools::Database;

#[derive(Database)]
#[database("pokemon_db")]
pub struct PokemonDb(sqlx::PgPool);
