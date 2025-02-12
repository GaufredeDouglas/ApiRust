use rocket::serde::json::Json;
use rocket_db_pools::Connection;
use crate::db::PokemonDb;
use crate::models::Pokemon;
use rocket::http::Status;
use rocket::response::status::Custom;
use sqlx::Acquire;

#[get("/pokemons?<page>&<per_page>")]
pub async fn get_pokemons(
    mut db: Connection<PokemonDb>, 
    page: Option<i64>, 
    per_page: Option<i64>
) -> Result<Json<Vec<Pokemon>>, Custom<String>> {
    let page = page.unwrap_or(1);
    let per_page = per_page.unwrap_or(10);
    let offset = (page - 1) * per_page;

    let pokemons: Vec<Pokemon> = sqlx::query_as::<_, Pokemon>(
        r#"
        SELECT 
            p.id,
            ps.identifier,
            ps.generation_id,
            ps.evolves_from_species_id,
            ps.evolution_chain_id,
            ps.color_id,
            ps.shape_id,
            ps.habitat_id,
            ps.gender_rate,
            ps.capture_rate,
            ps.base_happiness,
            ps.is_baby,
            ps.hatch_counter,
            ps.has_gender_differences,
            ps.growth_rate_id,
            ps.forms_switchable,
            ps.order,
            ps.conquest_order,
            p.height,
            p.weight, 
            p.base_experience,
            p.is_default
        FROM 
            pokemon p
        JOIN 
            pokemon_species ps ON p.species_id = ps.id
        ORDER BY 
            p.id
        LIMIT $1 OFFSET $2
        "#
    )
    .bind(per_page)
    .bind(offset)
    .fetch_all(&mut **db)
    .await
    .map_err(|e| Custom(Status::InternalServerError, format!("Database error: {}", e)))?;

    Ok(Json(pokemons))
}

#[get("/pokemons/<id>")]
pub async fn get_pokemon(mut db: Connection<PokemonDb>, id: i32) -> Result<Json<Pokemon>, Custom<String>> {
    let pokemon: Pokemon = sqlx::query_as::<_, Pokemon>(
        r#"
        SELECT 
            p.id,
            ps.identifier,
            ps.generation_id,
            ps.evolves_from_species_id,
            ps.evolution_chain_id,
            ps.color_id,
            ps.shape_id,
            ps.habitat_id,
            ps.gender_rate,
            ps.capture_rate,
            ps.base_happiness,
            ps.is_baby,
            ps.hatch_counter,
            ps.has_gender_differences,
            ps.growth_rate_id,
            ps.forms_switchable,
            ps.order,
            ps.conquest_order,
            p.height,
            p.weight, 
            p.base_experience,
            p.is_default
        FROM 
            pokemon p
        JOIN 
            pokemon_species ps ON p.species_id = ps.id
        WHERE p.id = $1
        "#
    )
    .bind(id)
    .fetch_one(&mut **db)
    .await
    .map_err(|e| Custom(Status::NotFound, format!("Pokemon not found: {}", e)))?;

    Ok(Json(pokemon))
}

// La séquence pour l'insert ne fonctionne pas donc on régénère la bonne valeur

#[post("/pokemons", data = "<pokemon>")]
pub async fn create_pokemon(mut db: Connection<PokemonDb>, pokemon: Json<Pokemon>) -> Result<Json<Pokemon>, Custom<String>> {
    let mut tx = db.begin().await
        .map_err(|e| Custom(Status::InternalServerError, format!("Failed to start transaction: {}", e)))?;

    sqlx::query("SELECT setval('pokemon_species_id_seq', (SELECT MAX(id) FROM pokemon_species))")
        .execute(&mut *tx)
        .await
        .map_err(|e| Custom(Status::InternalServerError, format!("Failed to reset pokemon_species sequence: {}", e)))?;

    sqlx::query("SELECT setval('pokemon_id_seq', (SELECT MAX(id) FROM pokemon))")
        .execute(&mut *tx)
        .await
        .map_err(|e| Custom(Status::InternalServerError, format!("Failed to reset pokemon sequence: {}", e)))?;

    let id: i32 = sqlx::query_scalar(
        r#"
        INSERT INTO pokemon_species (
            identifier, generation_id, evolves_from_species_id, evolution_chain_id, 
            color_id, shape_id, habitat_id, gender_rate, capture_rate, base_happiness, 
            is_baby, hatch_counter, has_gender_differences, growth_rate_id, 
            forms_switchable, "order", conquest_order
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
        RETURNING id
        "#
    )
    .bind(&pokemon.identifier)
    .bind(pokemon.generation_id)
    .bind(pokemon.evolves_from_species_id)
    .bind(pokemon.evolution_chain_id)
    .bind(pokemon.color_id)
    .bind(pokemon.shape_id)
    .bind(pokemon.habitat_id)
    .bind(pokemon.gender_rate)
    .bind(pokemon.capture_rate)
    .bind(pokemon.base_happiness)
    .bind(pokemon.is_baby)
    .bind(pokemon.hatch_counter)
    .bind(pokemon.has_gender_differences)
    .bind(pokemon.growth_rate_id)
    .bind(pokemon.forms_switchable)
    .bind(pokemon.order)
    .bind(pokemon.conquest_order)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| Custom(Status::InternalServerError, format!("Failed to insert pokemon species: {}", e)))?;

    let new_pokemon: Pokemon = sqlx::query_as::<_, Pokemon>(
        r#"
        WITH inserted_pokemon AS (
            INSERT INTO pokemon (identifier, species_id, height, weight, base_experience, "order", is_default)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, species_id, height, weight, base_experience, "order", is_default
        )
        SELECT 
            p.id, ps.identifier, ps.generation_id, ps.evolves_from_species_id, ps.evolution_chain_id,
            ps.color_id, ps.shape_id, ps.habitat_id, ps.gender_rate, ps.capture_rate, ps.base_happiness,
            ps.is_baby, ps.hatch_counter, ps.has_gender_differences, ps.growth_rate_id, ps.forms_switchable,
            ps.order, ps.conquest_order, p.height, p.weight, p.base_experience, p.is_default
        FROM 
            inserted_pokemon p
        JOIN 
            pokemon_species ps ON p.species_id = ps.id
        "#
    )
    .bind(&pokemon.identifier)
    .bind(id)
    .bind(pokemon.height)
    .bind(pokemon.weight)
    .bind(pokemon.base_experience)
    .bind(pokemon.order)
    .bind(pokemon.is_default)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| Custom(Status::InternalServerError, format!("Failed to create pokemon: {}", e)))?;

    tx.commit().await
        .map_err(|e| Custom(Status::InternalServerError, format!("Failed to commit transaction: {}", e)))?;

    Ok(Json(new_pokemon))
}


#[put("/pokemons/<id>", data = "<pokemon>")]
pub async fn update_pokemon(mut db: Connection<PokemonDb>, id: i32, pokemon: Json<Pokemon>) -> Result<Json<Pokemon>, Custom<String>> {
    let mut tx = db.begin().await
        .map_err(|e| Custom(Status::InternalServerError, format!("Failed to start transaction: {}", e)))?;

    let updated_pokemon = sqlx::query_as::<_, Pokemon>(
        r#"
        WITH updated_pokemon AS (
            UPDATE pokemon
            SET identifier = $1, height = $2, weight = $3, base_experience = $4, "order" = $5, is_default = $6
            WHERE id = $7
            RETURNING id, species_id, identifier, height, weight, base_experience, "order", is_default
        )
        UPDATE pokemon_species ps
        SET identifier = $8, generation_id = $9, evolves_from_species_id = $10, evolution_chain_id = $11,
            color_id = $12, shape_id = $13, habitat_id = $14, gender_rate = $15, capture_rate = $16,
            base_happiness = $17, is_baby = $18, hatch_counter = $19, has_gender_differences = $20,
            growth_rate_id = $21, forms_switchable = $22, "order" = $23, conquest_order = $24
        FROM updated_pokemon up
        WHERE ps.id = up.species_id
        RETURNING 
            up.id, ps.identifier, ps.generation_id, ps.evolves_from_species_id, ps.evolution_chain_id,
            ps.color_id, ps.shape_id, ps.habitat_id, ps.gender_rate, ps.capture_rate, ps.base_happiness,
            ps.is_baby, ps.hatch_counter, ps.has_gender_differences, ps.growth_rate_id, ps.forms_switchable,
            ps.order, ps.conquest_order, up.height, up.weight, up.base_experience, up.is_default
        "#
    )
    .bind(&pokemon.identifier)
    .bind(pokemon.height)
    .bind(pokemon.weight)
    .bind(pokemon.base_experience)
    .bind(pokemon.order)
    .bind(pokemon.is_default)
    .bind(id)
    .bind(&pokemon.identifier)
    .bind(pokemon.generation_id)
    .bind(pokemon.evolves_from_species_id)
    .bind(pokemon.evolution_chain_id)
    .bind(pokemon.color_id)
    .bind(pokemon.shape_id)
    .bind(pokemon.habitat_id)
    .bind(pokemon.gender_rate)
    .bind(pokemon.capture_rate)
    .bind(pokemon.base_happiness)
    .bind(pokemon.is_baby)
    .bind(pokemon.hatch_counter)
    .bind(pokemon.has_gender_differences)
    .bind(pokemon.growth_rate_id)
    .bind(pokemon.forms_switchable)
    .bind(pokemon.order)
    .bind(pokemon.conquest_order)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| Custom(Status::NotFound, format!("Failed to update pokemon: {}", e)))?;

    tx.commit().await
        .map_err(|e| Custom(Status::InternalServerError, format!("Failed to commit transaction: {}", e)))?;

    Ok(Json(updated_pokemon))
}


#[delete("/pokemons/<id>")]
pub async fn delete_pokemon(mut db: Connection<PokemonDb>, id: i32) -> Result<Status, Custom<String>> {
    let mut tx = db.begin().await
        .map_err(|e| Custom(Status::InternalServerError, format!("Failed to start transaction: {}", e)))?;

    let deleted_rows = sqlx::query("DELETE FROM pokemon_types WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| Custom(Status::InternalServerError, format!("Failed to delete from pokemon_types table: {}", e)))?;

    let species_id: Option<i32> = sqlx::query_scalar("DELETE FROM pokemon WHERE id = $1 RETURNING species_id")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| Custom(Status::InternalServerError, format!("Failed to delete from pokemon table: {}", e)))?;

    if let Some(species_id) = species_id {
        sqlx::query("UPDATE pokemon_species SET evolves_from_species_id = NULL WHERE evolves_from_species_id = $1")
        .bind(species_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| Custom(Status::InternalServerError, format!("Failed to update dependent pokemon_species: {}", e)))?;

        sqlx::query("DELETE FROM pokemon_species WHERE id = $1")
            .bind(species_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| Custom(Status::InternalServerError, format!("Failed to delete from pokemon_species table: {}", e)))?;

        tx.commit().await
            .map_err(|e| Custom(Status::InternalServerError, format!("Failed to commit transaction: {}", e)))?;

        Ok(Status::NoContent)
    } else {
        Err(Custom(Status::NotFound, format!("Pokemon with id {} not found", id)))
    }
}
