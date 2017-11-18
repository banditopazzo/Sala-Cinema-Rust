
extern crate mongodb;
extern crate sala_cinema;
extern crate chrono;
extern crate futures;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_serde_json;
extern crate serde_json;
extern crate bytes;

use sala_cinema::mongodb_service::PrenotazioneCollection;
use sala_cinema::models::{Prenotazione, Posto};
use sala_cinema::networking::*;

use mongodb::{Client, ThreadedClient};
use mongodb::db::ThreadedDatabase;

use futures::{Sink, Stream};
use futures::future::Future;

use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;
use tokio_core::net::TcpStream;

use bytes::BytesMut;

// Use length delimited frames
use tokio_io::codec::length_delimited;
use tokio_io::{AsyncRead, AsyncWrite};

use serde_json::Value;
use tokio_serde_json::{ReadJson, WriteJson};

fn main() {

    //Connect to MongoDB
    let mongo = Client::connect("localhost", 27017)
        .expect("Failed to initialize standalone client.");

    //Get the collection from the Database
    let mongo_coll = mongo.db("RustProva").collection("prenotazione");

    //Initialize the collection service
    let coll = PrenotazioneCollection { mongo_coll };

    // Create the event loop that will drive this server
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    // Bind the server's socket
    let addr = "127.0.0.1:12345".parse().unwrap();
    let listener = TcpListener::bind(&addr, &handle).unwrap();

    // Pull out a stream of sockets for incoming connections
    let server = listener.incoming().for_each(|(sock, _)| {

        println!("Connected 1 user");

        let (reader, writer) = sock.split();

        // Delimit frames using a length header
        let length_delimited = length_delimited::FramedRead::new(reader);
        let length_delimited_wr = length_delimited::FramedWrite::new(writer);

        // Serialize frames with JSON - necessario "mut" per uso successivo
        let mut serialized= WriteJson::<_ ,Value>::new(length_delimited_wr);

        // Deserialize frames
        let deserialized = ReadJson::<_, Value>::new(length_delimited)
            .map_err(|e| println!("ERR: {:?}", e));

        // Spawn a concurrent task that prints all received messages to STDOUT
        handle.spawn(deserialized.for_each(move |msg| {
            println!("received something");

            //Trick mostruoso, ma necessario
            let serialized = &mut serialized;

            let messaggio_ricevuto: Result<Message, _> = serde_json::from_value(msg);
            match messaggio_ricevuto {
                Ok(messaggio) => println!("GOT: {:?}", messaggio),
                Err(e) => println!("Errore di tipo: {:?}", e.classify())
            }

            // Create new message and send to the client, simulating a response
            let msg = Message::Quit;
            let jsoned = serde_json::to_value(&msg).unwrap();
            let action = serialized.send(jsoned);
            action.wait();
            println!("SENDED: {:?}", msg);

            Ok(())

        }));

        Ok(())
    });

    // Spin up the server on the event loop
    core.run(server).unwrap();

}

//TODO: parse command line arguments
//TODO: separare i vari casi e agire di conseguenza
//TODO: gestire segnali (es SIGPIPE)
