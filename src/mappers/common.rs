use mongodb::bson::oid::ObjectId;
use sha2::{Sha512, Digest};
use hex;

/// Convierte un ObjectId de MongoDB (owned) a String (hex)
/// Retorna un String vacío si el ObjectId es None o inválido
pub fn object_id_to_string_or_empty(oid: Option<ObjectId>) -> String {
    oid.map(|id| id.to_hex()).unwrap_or_default()
}

/// Hashea un string usando SHA512 y retorna el hash en formato hexadecimal
pub fn sha512_hash(input: &str) -> String {
    let mut hasher = Sha512::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}
