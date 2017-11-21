use super::models::Prenotazione;

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    Success(String),
    Error(String),
    GetMap,
    Prenota(Prenotazione),
    Delete(String),
    Quit,
}