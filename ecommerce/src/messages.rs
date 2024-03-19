use actix::Message;

// Un mensaje para indicar la lectura de un pedido.
//
// Este mensaje se utiliza en el contexto de Actix para señalizar la acción
// de leer un pedido. No contiene datos adicionales.
#[derive(Message)]
#[rtype(result = "()")]
pub struct ReadOrder();

// Un mensaje para representar la recepción de un pedido.
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
