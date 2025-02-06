use serde::{Serialize, Deserialize};
use sqlx::FromRow;

// On part du principe que l'on n'effectue les opérations de création / mise à jour / suppression sur la table pokemon uniquement
// Cette table représente des véritables individus, issus d'une espèce (pokemon_species)

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Pokemon {
    pub id: Option<i32>,
    pub identifier: String,
    pub generation_id: i32,
    pub evolves_from_species_id: Option<i32>,
    pub evolution_chain_id: i32,
    pub color_id: Option<i32>,
    pub shape_id: Option<i32>,
    pub habitat_id: Option<i32>,
    pub gender_rate: i32,
    pub capture_rate: i32,
    pub base_happiness: i32,
    pub is_baby: bool,
    pub hatch_counter: i32,
    pub has_gender_differences: bool,
    pub growth_rate_id: i32,
    pub forms_switchable: bool,
    pub order: i32,
    pub conquest_order: Option<i32>,
    pub height: i32,
    pub weight: i32,
    pub base_experience: i32,
    pub is_default: bool,
}
