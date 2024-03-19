use crate::product::Product;
use std::sync::Condvar;
use tokio::sync::Mutex;

// Representa el estado compartido dentro de una conexión de tienda.
//
// Esta estructura almacena una lista de productos a entregar y una variable de condición
// para la sincronización entre hilos. Se utiliza para gestionar los productos que deben
// ser procesados por un store específico y para sincronizar la entrega de productos
// entre diferentes partes del sistema.
//
// Atributos:
// * `products_to_deliver`: Un vector de `Mutex<Product>`. Cada `Mutex` envuelve un `Product`,
//   permitiendo el acceso seguro y concurrente a cada producto.
// * `condvar`: Una `Condvar` para la sincronización entre hilos. Se utiliza para notificar
//   a los hilos cuando hay productos disponibles para procesar.
pub struct SharedState {
    pub products_to_deliver: Vec<Mutex<Product>>,
    pub condvar: Condvar,
}

impl SharedState {
    // Crea una nueva instancia de `SharedState`.
    //
    // Inicializa el vector de productos a entregar como vacío y crea una nueva
    // variable de condición para la sincronización.
    //
    // Retorna:
    // Una nueva instancia de `SharedState`.
    pub fn new() -> Self {
        SharedState {
            products_to_deliver: Vec::new(),
            condvar: Condvar::new(),
        }
    }
}
