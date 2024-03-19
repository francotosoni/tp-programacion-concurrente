use actix::prelude::*;
use orders_processor::{process_line, process_store_orders};
use std::path::Path;
use std::sync::Arc;
use std::{env, io};
use store::Store;
use tokio::io::{split, AsyncBufReadExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::sync::Mutex;

use crate::store_server::StoreServer;

mod messages;
mod orders_processor;
mod product;
mod store;
mod store_server;

// Implementa la lógica principal del servidor
#[actix_rt::main]
async fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    // Verifica si se proporcionó el puerto y el archivo como argumento
    let (port, file) = match args.len() {
        3 => (args[1].clone(),args[2].clone()), // Si hay tres argumentos, el segundo es el puerto y el tercero es el file
        _ => {
            eprintln!("Uso: cargo run <puerto> <orders_file.csv>");
            return Ok(());
        }
    };


    // Creo un listener
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;

    // Defino el path de los pedidos
    let file_path = Path::new(&format!("./{}", file)).to_owned();

    let store = Store::new();
    let store_addr = store.start();

    // Creo un canal para comunicar lo que voy leyendo con
    let (tx, mut rx) = mpsc::channel::<csv::StringRecord>(16);

    // Lanzo una task que se encarga de leer el archivo
    let processor_handle = tokio::spawn(async move {
        process_store_orders(file_path, tx).await.unwrap();
    });

    // Lanzo una task que se encarga de procesar la linea.
    let addr_store = store_addr.clone();
    let result = tokio::spawn(async move {
        //La linea la recibo por el canal
        while let Some(line) = rx.recv().await {
            //Proceso la linea
            let order = process_line(line).await;
            let _result = addr_store.send(order).await;
            /*if result.unwrap() {
                println!("\x1b[32mSe pudo tomar el pedido\x1b[0m \n");
            } else {
                println!("\x1b[31mNo se pudo tomar el pedido\x1b[0m \n");
            }*/
        }
    });

    println!("Espero una conexión");
    while let Ok((stream, _addr)) = listener.accept().await {
        println!("\x1b[31mConexión nueva entrante\x1b[0m");
        let addr_store = store_addr.clone();
        StoreServer::create(|ctx| {
            let (r, w) = split(stream);
            let write = Arc::new(Mutex::new(w));
            StoreServer::add_stream(
                tokio_stream::wrappers::LinesStream::new(BufReader::new(r).lines()),
                ctx,
            );
            StoreServer::new(write, addr_store)
        });
    }

    // Esperar a que el procesador de órdenes termine
    processor_handle.await.unwrap();
    result.await.unwrap();

    Ok(())
}
