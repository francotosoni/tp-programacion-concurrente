use serde::{Deserialize, Serialize};

// Representa un producto en el sistema.
//
// Esta estructura se utiliza para almacenar información sobre un producto,
// incluyendo su identificador, la cantidad solicitada y una lista de tiendas
// donde se ha intentado enviar el pedido.
//
// Atributos:
// * `id`: Identificador del producto, representado por un entero de 32 bits.
// * `amount`: Cantidad del producto solicitada, representada por un entero de 32 bits.
// * `stores`: Vector que contiene las tiendas a las cuales se ha
//   intentado enviar el pedido.
#[derive(Serialize, Deserialize, Debug)]
pub struct Product {
    pub id: i32,
    pub amount: i32,
    pub stores: Vec<String>,
}

impl Product {
    // Agrega el nombre de una tienda a la lista de tiendas del producto.
    //
    // Esta función se utiliza para añadir una tienda al vector de tiendas
    // donde se ha intentado enviar el pedido.
    //
    // Argumentos:
    // * `store`: tienda a añadir, representado por una cadena de texto (`String`).
    pub fn add_store(&mut self, store: String) {
        self.stores.push(store);
    }

    // Obtiene una lista de las tiendas donde se ha intentado enviar el pedido.
    //
    // Esta función devuelve un nuevo vector que contiene las tiendas
    // donde se ha intentado enviar el pedido del producto.
    //
    // Retorna:
    // Un vector de cadenas de texto (`Vec<String>`) que representa las tiendas.
    pub fn get_stores(&self) -> Vec<String> {
        self.stores.clone()
    }
}
