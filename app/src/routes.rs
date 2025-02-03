use rocket::serde::json::Json;
use rocket::serde::json::serde_json;
use rocket_db_pools::Connection;
use crate::db::PokemonDb;
use crate::models::Pokemon;
use sqlx::types::JsonValue;

#[get("/pokemons?<page>&<per_page>")]
pub async fn get_pokemons(mut db: Connection<PokemonDb>, page: Option<i64>, per_page: Option<i64>) -> Json<Vec<Pokemon>> {
    let page = page.unwrap_or(1);
    let per_page = per_page.unwrap_or(10);
    let offset = (page - 1) * per_page;

    let pokemons: Vec<Pokemon> = sqlx::query_as(
        "SELECT data FROM pokemons LIMIT $1 OFFSET $2"
    )
    .bind(per_page)
    .bind(offset)
    .fetch_all(&mut **db)
    .await
    .unwrap()
    .into_iter()
    .map(|row: (JsonValue,)| serde_json::from_value(row.0).unwrap())
    .collect();

    Json(pokemons)
}


#[get("/pokemons/<id>")]
pub async fn get_pokemon(mut db: Connection<PokemonDb>, id: i32) -> Option<Json<Pokemon>> {
    let pokemon: Option<Pokemon> = sqlx::query_as(
        "SELECT data FROM pokemons WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&mut **db)
    .await
    .ok()?
    .map(|row: (JsonValue,)| serde_json::from_value(row.0).unwrap());

    pokemon.map(Json)
}

use sqlx::postgres::PgRow;
use sqlx::Row;

#[post("/pokemons", data = "<pokemon>")]
pub async fn create_pokemon(mut db: Connection<PokemonDb>, pokemon: Json<Pokemon>) -> Json<Pokemon> {
    let json_data = serde_json::to_value(&pokemon.0).unwrap();
    let new_pokemon: Pokemon = sqlx::query(
        "INSERT INTO pokemons (data) VALUES ($1) RETURNING data"
    )
    .bind(json_data)
    .map(|row: PgRow| {
        let json_value: sqlx::types::JsonValue = row.get("data");
        serde_json::from_value(json_value).unwrap()
    })
    .fetch_one(&mut **db)
    .await
    .unwrap();

    Json(new_pokemon)
}



#[put("/pokemons/<id>", data = "<pokemon>")]
pub async fn update_pokemon(mut db: Connection<PokemonDb>, id: i32, pokemon: Json<Pokemon>) -> Option<Json<Pokemon>> {
    let json_data = serde_json::to_value(&pokemon.0).unwrap();
    let updated_pokemon: Option<Pokemon> = sqlx::query_as(
        "UPDATE pokemons SET data = $1 WHERE id = $2 RETURNING data"
    )
    .bind(json_data)
    .bind(id)
    .fetch_optional(&mut **db)
    .await
    .ok()?
    .map(|row: (JsonValue,)| serde_json::from_value(row.0).unwrap());

    updated_pokemon.map(Json)
}

#[delete("/pokemons/<id>")]
pub async fn delete_pokemon(mut db: Connection<PokemonDb>, id: i32) -> Option<()> {
    sqlx::query("DELETE FROM pokemons WHERE id = $1")
        .bind(id)
        .execute(&mut **db)
        .await
        .ok()?;
    Some(())
}
