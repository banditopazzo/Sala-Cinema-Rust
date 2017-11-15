use super::models::Prenotazione;

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    Quit,
    Delete(String),
    Prenota(Prenotazione),
    GetMap
}