use actix::Message;

use crate::store::Store;

// Mensaje para indicar la apertura de un archivo.
//
// Este mensaje se utiliza para notificar a un actor `Store` que debe abrir un archivo.
// Contiene la dirección del actor `Store` que manejará la apertura del archivo.
//
// Atributos:
// * `Addr<Store>`: Dirección del actor `Store` responsable de manejar la apertura del archivo.
#[derive(Message)]
#[rtype(result = "()")]
pub struct OpenFile(pub actix::Addr<Store>);

// Mensaje para indicar la lectura de un pedido.
//
// Este mensaje se utiliza en el contexto de Actix para señalizar la acción
// de leer un pedido. No contiene datos adicionales.
#[derive(Message)]
#[rtype(result = "()")]
pub struct ReadOrder();

// Mensaje para representar la recepción de un pedido.
//
// Este mensaje se utiliza en el contexto de Actix para representar un pedido
// recibido. Contiene el `id` del producto y la `cantidad` solicitada.
//
// Atributos:
// * `id`: Identificador del producto, representado por un entero de 32 bits.
// * `amount`: Cantidad del producto solicitada, representada por un entero de 32 bits.
//
// Retorna un `bool` como resultado, indicando si la operación de recepción fue exitosa.
#[derive(Message)]
#[rtype(result = "bool")]
pub struct ReceiveOrder {
    pub id: i32,
    pub amount: i32,
}

// Mensaje para indicar el bloqueo de un producto.
//
// Este mensaje se utiliza para notificar a un actor `Store` que debe bloquear
// un producto específico, en respuesta a un pedido recibido.
// Contiene el `id` del producto y la `cantidad` a bloquear.
//
// Atributos:
// * `id`: Identificador del producto a bloquear.
// * `amount`: Cantidad del producto a bloquear.
#[derive(Message)]
#[rtype(result = "()")]
pub struct BlockProduct {
    pub id: i32,
    pub amount: i32,
}
