use async_std::task;
use file_reader::read_and_process_file;
use rand::seq::SliceRandom;
use rand::Rng;
use shared_state::SharedState;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;
use store_connection::handle_store_connection;
use tokio::io;

mod file_reader;
mod messages;
mod product;
mod read_stores;
mod shared_state;
mod store_connection;

// Punto de entrada principal del programa.
//
// Esta función asincrónica coordina la lectura de archivos CSV de productos y tiendas,
// establece conexiones con las tiendas y asigna productos a estas tiendas de manera aleatoria.
//
// La función realiza las siguientes operaciones:
// 1. Lee los productos del archivo "pedidos.csv" y las tiendas del archivo "stores.csv".
// 2. Crea un estado compartido para cada tienda y lanza una tarea asincrónica para manejar
//    la conexión con cada tienda.
// 3. Asigna los productos leídos a las tiendas de manera aleatoria.
// 4. Espera a que todas las conexiones de tiendas completen su procesamiento.
//
// Retorna:
// Un `io::Result<()>` que indica el resultado de la ejecución del programa.
// Retorna `Ok(())` si el programa se ejecuta correctamente o un error en caso de fallos.
#[tokio::main]
async fn main() -> io::Result<()> {
    let file_path = Path::new("./pedidos.csv");
    let products = read_and_process_file(file_path).await?;
    println!("[E-COMMERCE] {} products read", products.len());

    let stores = Arc::new(read_stores::read_stores("./stores.csv").unwrap());
    let mut store_ids = Vec::new();
    let mut connections = Vec::new();
    let mut store_states = HashMap::new();
    
    // Crear la estructura compartida con Mutex y Condvar para cada tienda.
    for (id, address) in stores.iter() {
        store_ids.push(id.clone());
        let shared_state = Arc::new((Mutex::new(SharedState::new()), Condvar::new()));
        store_states.insert(id.clone(), shared_state.clone());
    }
    
    for (id, address) in stores.iter(){
        println!("[E-COMMERCE] Intentando conectar al Store {}: {}", id, address);
        let id_clone = id.clone();
        let address_clone = address.clone();
        let shared_state = Arc::clone(&store_states.get(id).unwrap());  
        let stores_id_clone = store_ids.clone();
        let stores_states_clone = store_states.clone();
        let connection = tokio::spawn(async move {
            handle_store_connection(
                id_clone,
                address_clone,
                shared_state,
                stores_id_clone,
                stores_states_clone,
            )
            .await;
        });
        connections.push(connection);
    }
    
    let mut rng = rand::thread_rng();
    // Asignar productos a las conexiones de manera aleatoria
    for product in products {
        if let Some(random_id) = store_ids.choose(&mut rng) {
            let store_id_str = random_id.to_string();
            if let Some(shared_state_arc) = store_states.get(&store_id_str) {
                let (shared_state_mutex, cvar) = &**shared_state_arc;
                let mut shared_state = shared_state_mutex.lock().unwrap();
                shared_state.products_to_deliver.push(product);
                cvar.notify_one();
            }
        }

        let sleep_time = rand::thread_rng().gen_range(1..5);
        task::sleep(Duration::from_secs(sleep_time)).await;
    }
    println!("No tengo mas productos para enviar");

    // Esperar a que todas las tareas asincrónicas se completen
    for connection in connections {
        let _ = connection.await;
    }

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn read_orders_csv() {
        let file_path = Path::new("./pedidos.csv");
        let products = match read_and_process_file(file_path).await {
            Ok(products) => products,
            Err(err) => {
                panic!("Failed to read and process file: {:?}", err);
            }
        };
        assert_eq!(
            products.len(),
            100
        );
    }

    #[tokio::test]
    async fn giving_orders_to_stores() {

        let file_path = Path::new("./pedidos.csv");
        let products = match read_and_process_file(file_path).await {
            Ok(products) => products,
            Err(err) => {
                panic!("Failed to read and process file: {:?}", err);
            }
        };
   
        let stores = Arc::new(read_stores::read_stores("./stores.csv").unwrap());
        let mut store_ids = Vec::new();
        let mut store_states = HashMap::new();
    
        // Crear la estructura compartida con Mutex y Condvar para cada tienda.
        for (id, address) in stores.iter() {
            store_ids.push(id.clone());
            let shared_state = Arc::new((Mutex::new(SharedState::new()), Condvar::new()));
            store_states.insert(id.clone(), shared_state.clone());
        }

        let mut rng = rand::thread_rng();
        // Asignar productos a las conexiones de manera aleatoria
        for product in products {
            if let Some(random_id) = store_ids.choose(&mut rng) {
                let store_id_str = random_id.to_string();
                if let Some(shared_state_arc) = store_states.get(&store_id_str) {
                    let (shared_state_mutex, cvar) = &**shared_state_arc;
                    let mut shared_state = shared_state_mutex.lock().unwrap();
                    shared_state.products_to_deliver.push(product);
                }
            }

            let sleep_time = rand::thread_rng().gen_range(1..5);
            task::sleep(Duration::from_secs(sleep_time)).await;
        }

        let (shared_state_mutex, cvar) = &**store_states.get("1").unwrap();
        let shared_state = shared_state_mutex.lock().unwrap();
        let mut orders = shared_state.products_to_deliver.len();

        let (shared_state_mutex, cvar) = &**store_states.get("2").unwrap();
        let shared_state = shared_state_mutex.lock().unwrap();
        orders += shared_state.products_to_deliver.len();

        assert_eq!(
            orders,
            100
        );
    }
}
