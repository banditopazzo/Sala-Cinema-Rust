use bson;
use chrono::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct Posto {
    pub fila: String,
    pub numero: i32
}

#[derive(Serialize,  Deserialize, Debug)]
pub struct Prenotazione {
    #[serde( skip_serializing_if = "Option::is_none")]
    pub codice: Option<bson::oid::ObjectId>,
    pub data: DateTime<Utc>,
    pub posto: Posto
}