
extern crate serde_json;
extern crate tokio_io;
extern crate tokio_serde_json;
extern crate futures;
extern crate tokio_core;
extern crate sala_cinema;
extern crate chrono;
extern crate bytes;


use chrono::prelude::*;

use sala_cinema::models::{Prenotazione, Posto};
use sala_cinema::networking::*;

use futures::{Future, Sink, Stream};
use futures::sync::mpsc;

use tokio_core::reactor::Core;
use tokio_core::net::TcpStream;

use tokio_io::AsyncRead;

// Use length delimited frames
use tokio_io::codec::length_delimited;
use tokio_serde_json::{ReadJson, WriteJson};
use serde_json::Value;

use std::io;
use std::thread;

fn main() {

    // Create the event loop that will drive this server
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    //Nuovo Thread e canale stdin
    let (stdin_tx, stdin_rx) = mpsc::channel(0);
    thread::spawn(|| read_stdin(stdin_tx));
    let stdin_rx = stdin_rx.map_err(|_| panic!()); // errors not possible on rx

    // Bind a server socket
    let socket = TcpStream::connect(
        &"127.0.0.1:12345".parse().unwrap(),
        &handle);

    let client = socket.and_then(move |sock| {

        let (reader, writer) = sock.split();

        // Delimit frames using a length header
        //Read
        let length_delimited = length_delimited::FramedRead::new(reader);
        let deserialized = ReadJson::<_, Value>::new(length_delimited)
            .map_err(|e| println!("ERR: {:?}", e));
        //Write
        let length_delimited_wr = length_delimited::FramedWrite::new(writer);
        let mut serialized= WriteJson::<_ ,Value>::new(length_delimited_wr);

        //Future that takes messages from the stdin-reader-thread and forward them to the socket
        handle.clone().spawn(stdin_rx.for_each(move |msg| {

            let serialized = &mut serialized;

            let jsoned = serde_json::to_value(&msg).unwrap();

            // Send the value
            match serialized.send(jsoned).wait() {
                Ok(_) => println!("SENDED: {:?}", msg),
                Err(_) => println!("Error sending message")
            };

            Ok(())
        }));

        //Future that takes messages from the socket and print them on the stout
        handle.clone().spawn(deserialized.for_each(|msg| {

            let messaggio_ricevuto: Result<Message, _> = serde_json::from_value(msg);
            match messaggio_ricevuto {
                Ok(messaggio) => println!("GOT: {:?}", messaggio),
                Err(e) => println!("Errore di tipo: {:?}", e.classify())
            }

            Ok(())

        }));

        Ok(())

    });

    let client = client.map_err(|_| panic!());
    core.handle().spawn(client);


    //Loop the event loop!! :)
    loop {
        core.turn(None);
    }

}

// Our helper method which will read data from stdin and send it along the
// sender provided.
fn read_stdin(mut tx: mpsc::Sender<Message>) {
    let mut stdin = io::stdin();
    loop {
        println!("\n\nSeleziona l'operazione\n\n");
        println!("1 - Mostra mappa dei posti disponibili\n");
        println!("2 - Effettua una prenotazione\n");
        println!("3 - Disdici una prenotazione conoscendo il codice\n");
        println!("4 - Esci dal programma\n\n");
        println!("Inserisci il numero corrispondente all'operazione scelta:  ");

        let mut guess = String::new();

        io::stdin().read_line(&mut guess)
            .expect("Failed to read line");

        let guess: u32 = match guess.trim().parse() {
            Ok(num) => num,
            Err(_) => continue,
        };

        //Convert option in message
        let msg = match guess {
            1 => Message::GetMap,
            2 => Message::Error(String::from("Not implemented yet")),
            3 => Message::Delete(String::from("prova")),
            4 => Message::Quit,
            _ => Message::Error(String::from("Opzione non riconosciuta. Riprovare..."))
        };

        //Check for unknown command
        match msg {
            Message::Error(msg) => {
                println!("{}", msg);
                continue;
            },
            _ => {}
        }

        //Send the message to the primary thread
        tx = match tx.send(msg).wait() {
            Ok(tx) => tx,
            Err(_) => break,
        };
    }
}




//TODO: parse command line arguments