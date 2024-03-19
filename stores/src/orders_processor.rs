// store_orders_processor.rs
use csv::ReaderBuilder;
use rand::Rng;
use std::io;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc;

use crate::messages::ReceiveOrder;

pub async fn process_line(line: csv::StringRecord) -> ReceiveOrder {
    // Simula el procesamiento de la línea
    println!("[LINE PROCESS] Procesando línea: {:?}", line);
    // TODO: lógica para procesar línea del CSV
    ReceiveOrder {
        id: line.get(0).unwrap().parse::<i32>().unwrap(),
        amount: line.get(1).unwrap().parse::<i32>().unwrap(),
    }
}

pub async fn process_store_orders(
    file_path: PathBuf,
    tx: mpsc::Sender<csv::StringRecord>,
) -> io::Result<()> {
    let mut rdr = ReaderBuilder::new().from_path(file_path)?;

    for result in rdr.records() {
        let record = result?;
        let sleep_time = rand::thread_rng().gen_range(1..5);
        tokio::time::sleep(Duration::from_secs(sleep_time)).await;
        tx.send(record).await.unwrap();
    }

    Ok(())
}
