use crate::product::Product;
use std::path::Path;
use std::sync::Arc;
use std::vec::Vec;
use tokio::fs::File;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::sync::Mutex;

// Función para procesar una línea del archivo.
//
// Asume que cada línea del archivo está en el formato "id,cantidad".
// Descompone la línea en sus componentes y crea un producto.
//
// Argumentos:
// * `line`: Una línea del archivo como `String`.
//
// Retorna:
// Un `io::Result<Product>` que es `Ok` con un `Product` si la línea se procesa correctamente,
// o un error en caso contrario.
async fn process_line(line: String) -> io::Result<Product> {
    let parts: Vec<&str> = line.split(',').collect();
    let id = parts[0].parse::<i32>().unwrap();
    let amount = parts[1].parse::<i32>().unwrap();
    Ok(Product {
        id,
        amount,
        stores: Vec::new(),
    })
}

// Lee y procesa un archivo para crear una lista de productos.
//
// Abre el archivo especificado en `file_path`, lo lee línea por línea,
// y procesa cada línea para crear un `Product`.
// La primera línea del archivo se asume que es un encabezado y se omite.
// Cada producto se encapsula en un `Mutex` para un manejo seguro en un entorno concurrente.
//
// Argumentos:
// * `file_path`: Una referencia a un `Path` que representa la ruta del archivo a leer.
//
// Retorna:
// Un `io::Result<Vec<Mutex<Product>>>` que es `Ok` con un vector de `Mutex<Product>` si el archivo
// se lee y procesa correctamente, o un error en caso contrario.
pub async fn read_and_process_file(file_path: &Path) -> io::Result<Vec<Mutex<Product>>> {
    let file = File::open(file_path).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    lines.next_line().await?;

    let products = Arc::new(Mutex::new(Vec::new()));

    let mut tasks = vec![];

    while let Some(line) = lines.next_line().await? {
        let products_clone = Arc::clone(&products);
        let task = tokio::spawn(async move {
            let product = process_line(line).await.unwrap();
            let mut products = products_clone.lock().await;
            products.push(Mutex::new(product));
        });
        tasks.push(task);
    }

    for task in tasks {
        task.await?;
    }

    let final_products = Arc::try_unwrap(products)
        .expect("Arc still has multiple owners")
        .into_inner();
    Ok(final_products)
}
