use crate::messages::{BlockProduct, ReceiveOrder};
use crate::product::Product;
use actix::{Actor, Addr, Context, StreamHandler};
use serde_json::{self};
use std::sync::Arc;
use std::io;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::Store;
// Definición del actor StoreServer
// Representa la lógica para manejar una conexión de cliente.
pub struct StoreServer {
    write: Arc<Mutex<tokio::io::WriteHalf<TcpStream>>>,
    store_addr: Addr<Store>,
}

impl StoreServer {
    pub fn new(
        write: Arc<Mutex<tokio::io::WriteHalf<TcpStream>>>,
        store_addr: Addr<Store>,
    ) -> StoreServer {
        StoreServer {
            write,
            store_addr,
        }
    }
}

impl Actor for StoreServer {
    type Context = Context<Self>;
}

// Implementa el manejo de los mensajes entrantes
impl StreamHandler<Result<String, io::Error>> for StoreServer {
    fn handle(&mut self, msg: Result<String, io::Error>, _ctx: &mut Self::Context) {
        // Aquí manejas los mensajes entrantes, por ejemplo, pedidos de e-commerce
        let pedido = msg.expect("Error cuando se recibe el mensaje");
        println!("[ACTOR STORE SERVER] Recibi un mensaje: {}", pedido);
        //Suponiendo que recibo una linea de texto formato id,cantidad
        // Deserializar JSON a la estructura Product
        match serde_json::from_str::<Product>(&pedido) {
            Ok(product) => {
                // Ahora tenes una instancia de Product
                println!(
                    "[ACTOR STORE SERVER] ID: {}, Amount: {}",
                    product.id, product.amount
                );

                let order = ReceiveOrder {
                    id: product.id,
                    amount: product.amount,
                };
                let store_addr = self.store_addr.clone();
                //Se agrego el spawn de esta task porque necesitaba esperar por la respuesta de si se encontraba disponible o no
                //el producto para bloquearlo y derivarlo al delivery
                let write_guard = self.write.clone();
                tokio::spawn(async move {
                    let send_result = store_addr.send(order).await;
                    if send_result.unwrap() {
                        let block_result: Result<(), actix::prelude::MailboxError> = store_addr
                            .send(BlockProduct {
                                id: product.id,
                                amount: product.amount,
                            })
                            .await;
                        match block_result {
                            Ok(()) => {
                                println!("[ACTOR STORE SERVER] Pedido bloqueado exitosamente");
                                //Mando al ecommerce que puedo tomar el pedido
                                let response: u8 = true as u8;
                                write_guard
                                    .lock()
                                    .await
                                    .write_u8(response)
                                    .await
                                    .expect("Se tuvo que mandar");
                            }
                            Err(mailbox_error) => {
                                println!(
                                    "\x1b[31m[ACTOR STORE SERVER] Error al enviar el mensaje para bloquear el producto: {}\x1b[0m",
                                    mailbox_error
                                );
                            }
                        }
                    } else {
                        //Mando al ecommerce que no puedo tomar el pedido
                        let response: u8 = false as u8;
                        write_guard
                            .lock()
                            .await
                            .write_u8(response)
                            .await
                            .expect("Se tuvo que mandar");
                    }
                });
            }
            Err(e) => {
                eprintln!(
                    "\x1b[31m[ACTOR STORE SERVER] Error al deserializar el mensaje: {}\x1b[0m",
                    e
                );
            }
        }
    }

    fn finished(&mut self, _ctx: &mut Self::Context) {
        // Maneja la desconexión
        println!("\x1b[31m[ACTOR STORE SERVER] Se desconecto\x1b[0m");
    }
}
/*
// Función para procesar pedidos del archivo de manera asincrónica
async fn process_store_orders(file_path: &Path) -> io::Result<()> {
    // ... [Código para procesar pedidos del archivo] ...
}
*/

// Aquí puedes añadir más funciones, como la lectura y procesamiento de pedidos de un archivo
