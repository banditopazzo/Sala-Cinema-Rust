
extern crate mongodb;
extern crate sala_cinema;
extern crate chrono;
extern crate futures;
extern crate futures_cpupool;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_serde_json;
extern crate serde_json;
extern crate bytes;

use std::error::Error;

use sala_cinema::mongodb_service::PrenotazioneCollection;
use sala_cinema::models::{Prenotazione, Posto};
use sala_cinema::networking::*;

use mongodb::{Client, ThreadedClient};
use mongodb::db::ThreadedDatabase;

use futures::{Sink, Stream};
use futures::future::{Executor,Future};
use futures::sync::mpsc;
use futures::sync::mpsc::{Sender, Receiver};
use futures_cpupool::CpuPool;


use tokio_core::net::TcpListener;
use tokio_core::reactor::{Handle, Core};
use tokio_core::net::TcpStream;

use bytes::BytesMut;

// Use length delimited frames
use tokio_io::codec::length_delimited;
use tokio_io::AsyncRead;

use serde_json::Value;
use tokio_serde_json::{ReadJson, WriteJson};

use std::thread;

fn main() {

    const DB_NAME: &str = "RustProva";
    const COLLECTION_NAME: &str = "prenotazione";
    const NUM_THREADS: i32 = 4;

    //Connect to MongoDB
    let mongo = Client::connect("localhost", 27017)
        .expect("Failed to initialize standalone client.");

    // Create the event loop that will drive this server
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    // set up a thread pool
    let pool = CpuPool::new_num_cpus();

    // Bind the server's socket
    let addr = "127.0.0.1:12345".parse().unwrap();
    let listener = TcpListener::bind(&addr, &handle).unwrap();

    // Pull out a stream of sockets for incoming connections
    let server = listener.incoming().for_each(|(sock, address)| {

        println!("Connected 1 user. Addr: {:?}", address);

        let (reader, writer) = sock.split();

        // Delimit frames using a length header
        let length_delimited = length_delimited::FramedRead::new(reader);
        let length_delimited_wr = length_delimited::FramedWrite::new(writer);

        // Serialize frames with JSON - necessario "mut" per uso successivo
        let mut serialized= WriteJson::<_ ,Value>::new(length_delimited_wr);

        // Deserialize frames
        let deserialized = ReadJson::<_, Value>::new(length_delimited)
            .map_err(|e| println!("ERR: {:?}", e));

        //Clone db
        let mongo = mongo.clone();

        //Channel between the main thread and the future pool
        let (sender, mut receiver) = mpsc::channel(0);

        // Spawn a concurrent task that wait for future response and send result to the client
        handle.spawn(receiver.for_each( move |response| {

            let serialized = &mut serialized;

            //Send response to client
            match serialized.send(response).wait() {
                Ok(_) => println!("SENDED: to client"),
                Err(_) => println!("Error sending message")
            };

            Ok(())

        }));

        // Execute the future in the pool
        pool.execute(deserialized.for_each(move |msg| {

            //Set-up DB service
            let mongo_coll = mongo.clone().db(DB_NAME).collection(COLLECTION_NAME);
            let db_service = PrenotazioneCollection { mongo_coll };

            let messaggio_ricevuto: Result<Message, _> = serde_json::from_value(msg);
            let messaggio_ricevuto = match messaggio_ricevuto {
                Ok(messaggio) => messaggio,
                Err(e) => Message::Error(String::from("Messaggio non riconosciuto"))
            };

            let response = match messaggio_ricevuto {
                Message::GetMap => Message::Quit,
                Message::Prenota(posto) => Message::Quit,
                Message::Delete(id) => elimina_posto(&db_service, &id),
                Message::Quit => Message::Quit,
                Message::Error(err) => Message::Error(err),
                _ => Message::Error(String::from("Messaggio non accettato"))
            };

            let sender = sender.clone();
            let jsoned = serde_json::to_value(&response).unwrap();
            match sender.send(jsoned).wait() {
                Ok(_) => println!("SENDED response to main thread"),
                Err(e) => println!("Error sending message to the main thread: {:?}", e.description())
            };

            Ok(())

        }));

        Ok(())
    });

    // Spin up the server on the event loop
    core.run(server).unwrap();

}

fn elimina_posto(coll: &PrenotazioneCollection, id: &String) -> Message {
    match coll.delete_by_codice(&id) {
        Ok(_) => Message::Success(String::from("Prenotazione eliminata")),
        Err(e) => Message::Error(String::from(e.description()))
    }
}

//TODO: parse command line arguments
//TODO: separare i vari casi e agire di conseguenza
//TODO: gestire segnali (es SIGPIPE)
