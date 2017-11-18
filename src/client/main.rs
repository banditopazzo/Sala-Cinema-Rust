
extern crate serde_json;
extern crate tokio_io;
extern crate tokio_serde_json;
extern crate futures;
extern crate tokio_core;
extern crate sala_cinema;
extern crate chrono;
extern crate bytes;


use bytes::BytesMut;

use chrono::prelude::*;

use sala_cinema::models::{Prenotazione, Posto};
use sala_cinema::networking::*;

use futures::{Future, Sink, Stream};
use futures::future::Executor;
use futures::sync::mpsc;

use tokio_core::reactor::{Core, Handle};
use tokio_core::net::TcpStream;

use tokio_io::{AsyncRead, AsyncWrite};
use tokio_io::io::{ReadHalf, WriteHalf};

// Use length delimited frames
use tokio_io::codec::{length_delimited, FramedWrite, FramedRead};
use tokio_serde_json::{ReadJson, WriteJson};
use serde_json::Value;

use std::io;
use std::thread;

/*struct IO_Container{
    reader: ReadJson<FramedRead<ReadHalf<TcpStream>, BytesMut>, Value>,
    writer: WriteJson<FramedWrite<WriteHalf<TcpStream>, BytesMut>, Value>
}*/

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

        handle.clone().spawn(stdin_rx.for_each(move |msg|{
            println!("readed something from keyboard");

            let serialized = &mut serialized;

            let msg = Message::Quit;
            let jsoned = serde_json::to_value(&msg).unwrap();

            // Send the value
            let action = serialized.send(jsoned);
            action.wait();

            println!("SENDED: {:?}", msg);
            Ok(())
        }));



        handle.clone().spawn(deserialized.for_each(|msg| {

            let messaggio_ricevuto: Result<Message, _> = serde_json::from_value(msg);
            match messaggio_ricevuto {
                Ok(messaggio) => println!("GOT: {:?}", messaggio),
                Err(e) => println!("Errore di tipo: {:?}", e.classify())
            }

            Ok(())

        }));

        println!("Spawned futures");

        Ok(())

    });

    let client = client.map_err(|_| panic!());
    core.handle().spawn(client);


    println!("before loop");
    //loop provvisoriomap_err(|_| panic!());
    loop {
        core.turn(None);
    }

}

// Our helper method which will read data from stdin and send it along the
// sender provided.
fn read_stdin(mut tx: mpsc::Sender<Message>) {
    let mut stdin = io::stdin();
    loop {
        println!("Please enter your choice.");

        let mut guess = String::new();

        io::stdin().read_line(&mut guess)
            .expect("Failed to read line");

        let guess: u32 = match guess.trim().parse() {
            Ok(num) => num,
            Err(_) => continue,
        };

        println!("You choose: {}", guess);

        match guess {
            1 => println!("OK"),
            _ => continue
        }

        let msg = Message::Quit;

        tx = match tx.send(msg).wait() {
            Ok(tx) => tx,
            Err(_) => break,
        };
    }
}




//TODO: parse command line arguments
//TODO: event loop bloccante for user input