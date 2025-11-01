use mongodb::bson::oid::ObjectId;

/// Convierte un ObjectId de MongoDB (owned) a String (hex)
/// Retorna un String vacío si el ObjectId es None o inválido
pub fn object_id_to_string_or_empty(oid: Option<ObjectId>) -> String {
    oid.map(|id| id.to_hex()).unwrap_or_default()
}

