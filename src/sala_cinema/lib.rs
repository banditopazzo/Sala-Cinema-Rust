#[macro_use(bson, doc)]
extern crate bson;
extern crate mongodb;
extern crate chrono;

#[macro_use]
extern crate serde_derive;

pub mod models;

pub mod networking;

pub mod mongodb_service;