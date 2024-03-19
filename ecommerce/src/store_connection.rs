use crate::shared_state::SharedState;
use async_std::task;
use serde_json::to_string;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

// Maneja la conexión a un store y procesa los productos asignados.
//
// Esta función establece una conexión TCP con un store específico y procesa productos
// asignados a este store. Intenta reconectar si la conexión se pierde y maneja la
// lógica de reasignación de productos si el store actual no puede procesarlos.
//
// Argumentos:
// * `id`: El identificador del store, representado por una cadena de texto (`String`).
// * `address`: La dirección IP del store, también como una cadena de texto.
// * `shared_state`: Un `Arc` conteniendo un `Mutex` que envuelve el estado compartido (`SharedState`)
//   y una `Condvar` para la sincronización de hilos.
// * `stores_ids`: Vector de identificadores de tiendas disponibles.
// * `stores_states`: Mapa que asocia los identificadores de tiendas con sus respectivos estados compartidos.
//
// La función entra en un bucle infinito, manejando la conexión TCP y procesando productos.
// Dentro del bucle, se maneja la conexión y, si es exitosa, se procesan los productos asignados
// al store. Si la conexión falla, se realiza un intento de reconexión después de un período de espera.
// En el procesamiento de productos, si un store no puede manejar un producto (por ejemplo, falta de stock),
// se busca otro store y se reasigna el producto.
pub async fn handle_store_connection(
    id: String,
    address: String,
    shared_state: Arc<(Mutex<SharedState>, Condvar)>,
    stores_ids: Vec<String>,
    stores_states: HashMap<String, Arc<(Mutex<SharedState>, Condvar)>>,
) {
    loop {
        match TcpStream::connect(&address).await {
            Ok(mut stream) => {
                println!(
                    "[E-COMMERCE] \x1b[32m[Store {}] Conexión exitosa al store: {}\x1b[0m",
                    id, address
                );
                loop {
                    let product = {
                        let (lock, cvar) = &*shared_state;
                        let mut state = lock.lock().unwrap();
                        while state.products_to_deliver.is_empty() {
                            state = cvar.wait(state).unwrap();
                        }
                        state.products_to_deliver.pop()
                        //Some(state.products_to_deliver.remove(0))
                    };

                    if let Some(product) = product {
                        println!("\n[E-COMMERCE] [Store {}] Processing product {:?}", id,product);
                        task::sleep(Duration::from_secs(5)).await;
                        let serialized_product = to_string(&*product.lock().await).unwrap();

                        if let Err(e) = stream
                            .write_all((serialized_product.clone() + "\n").as_bytes())
                            .await
                        {
                            if e.kind() == ErrorKind::BrokenPipe {
                                let (lock, _cvar) = &*shared_state;
                                let mut state = lock.lock().unwrap();
                                state.products_to_deliver.push(product);
                            }
                            eprintln!("[E-COMMERCE] \x1b[31m[Store {}] Error al enviar datos: {}\x1b[0m", id, e);
                            break; // Sale de la función si hay un error
                        } else {
                            println!(
                                "[E-COMMERCE] \x1b[32m[Store {}] Producto enviado exitosamente: {}\x1b[0m",
                                id, serialized_product
                            );
                        }
                        let response = stream.read_u8().await;
                        match response {
                            Ok(r) => {
                                if r == 0 {
                                    println!("[E-COMMERCE] \x1b[34m[Store {}] No se encuentra stock en el local pedido. Pido en otro\x1b[0m", id);
                                    let id_cloned = id.clone();
                                    product.lock().await.add_store(id_cloned);
                                    //Busco un nuevo local
                                    let product_stores = product.lock().await.get_stores();
                                    let result = stores_ids.iter().find(|&key| !product_stores.contains(key));
                                    match result {
                                        Some(store) => {
                                            let (shared_state_mutex, cvar) = &**stores_states.get(store).unwrap();
                                            let mut shared_state = shared_state_mutex.lock().unwrap();
                                            shared_state.products_to_deliver.push(product);
                                            cvar.notify_one();
                                        },
                                        None => println!("[E-COMMERCE] \x1b[31m[Store {}] No hay mas stores disponibles.\x1b[0m", id),
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("[E-COMMERCE] \x1b[31m[Store {}] Error al leer la respuesta del store: {}\x1b[0m", id, e);
                                //return;
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "[E-COMMERCE] \x1b[31m[Store {}] Error al intentar conectar al store: {}\x1b[0m \n",
                    id, e
                );
                task::sleep(Duration::from_secs(10)).await; // Esperar antes de intentar nuevamente
            }
        }
    }
}
