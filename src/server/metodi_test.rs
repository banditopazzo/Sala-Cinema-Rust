
fn scrivi(coll: &PrenotazioneCollection) {
    let dt = Utc.ymd(2017,6,3).and_hms(0,0,0);
    let pr = Prenotazione {
        codice: None,
        data: dt,
        posto: Posto {
            fila: String::from("XX"),
            numero: 66
        }
    };
    println!("{:?}", pr);

    match coll.write(&pr) {
        Ok(_) => println!("Scrittura eseguita correttamente"),
        Err(why) => println!("{:?}", why)
    }
}

fn leggi(coll: &PrenotazioneCollection) {
    let document = coll.find_by_codice(&String::from("5a0873a1373862ae24490752"));

    match document {
        Ok(Some(doc)) => println!("{:?}", doc),
        Ok(None) => panic!("no document found"),
        Err(why) => panic!("{:?}", why)
    }
}

fn cancella(coll: &PrenotazioneCollection) {
    let result = coll.delete_by_codice(&String::from("5a0873a1373862ae24490752"));

    match result {
        Ok(res) => println!("Cancellato: {:?}", res.acknowledged),
        Err(_) => panic!("errore durante la cancellazione")
    }
}