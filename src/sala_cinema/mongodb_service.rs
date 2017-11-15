use mongodb;
use bson;
use super::models::Prenotazione;
use chrono::prelude::*;

pub struct PrenotazioneCollection {
    pub mongo_coll: mongodb::coll::Collection
}

impl PrenotazioneCollection {

    pub fn write(&self, pr: &Prenotazione) -> Result<mongodb::coll::results::InsertOneResult,mongodb::Error> {

        let serialized = bson::to_bson(&pr)?;

        if let bson::Bson::Document(document) = serialized {
            self.mongo_coll.insert_one(document, None)// Insert into a MongoDB collection
        } else {
            Err(mongodb::Error::ArgumentError(String::from("Error converting the BSON object into a MongoDB document")))
        }

    }

    pub fn find_by_codice(&self, codice: &String) -> Result<Option<bson::ordered::OrderedDocument>, mongodb::Error> {

        let codice = bson::oid::ObjectId::with_string(codice);

        match codice {
            Ok(id) => self.mongo_coll.find_one(Some(doc! { "_id": id }), None),
            Err(_) => Err(mongodb::Error::ArgumentError(String::from("Error creating the ObjectID from string")))
        }

    }

    pub fn find_by_data(&self, data: &DateTime<Utc>) -> Result<mongodb::cursor::Cursor, mongodb::Error>{

        let serialized = bson::to_bson(&data)?;

        if let bson::Bson::Document(document) = serialized {
            self.mongo_coll.find(Some(doc! { "data" : document}), None)
        } else {
            Err(mongodb::Error::ArgumentError(String::from("Error converting the BSON object into a MongoDB document")))
        }

    }

    pub fn delete_by_codice(&self, codice: &String) -> Result<mongodb::coll::results::DeleteResult, mongodb::Error> {

        let codice = bson::oid::ObjectId::with_string(codice);

        match codice {
            Ok(id) => self.mongo_coll.delete_one(doc! { "_id": id }, None),
            Err(_) => Err(mongodb::Error::ArgumentError(String::from("Error creating the ObjectID from string")))
        }

    }

}