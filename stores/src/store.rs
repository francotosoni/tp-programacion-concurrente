use std::{
    collections::HashMap,
    sync::{Arc, Condvar, Mutex},
    thread,
    time::Duration,
};

use crate::messages::{BlockProduct, ReceiveOrder};
use crate::product::Product;
use actix::{Actor, Context, Handler};
use rand::{
    distributions::{Bernoulli, Distribution},
    Rng,
};

use rand::thread_rng;

// Constante para calcular si se entrego o no un pedido
const PROBABILITY_OF_SUCCESS_DELIVERY: f64 = 0.8;

// Constante para determinar la cantidad de procesos dedicados a realizar el delivery
const AMAOUNT_OF_DELIVERY_PROCESS: u32 = 5;

pub struct Store {
    products: Arc<Mutex<HashMap<i32, Product>>>,
    orders_blocked: Arc<Mutex<Vec<Product>>>, //Productos bloqueados para ser retirados
    condv_orders: Arc<Condvar>, //Vamos a estar notificando a los procesos cuando se ponga un nuevo producto para hacer delivery
    delivery_process: Vec<thread::JoinHandle<()>>, //Pool de threads encargados de hacer el delivery
    bernoulli_dist: Bernoulli,
}

impl Store {
    pub fn new() -> Store {
        let mut store = Store {
            products: Arc::new(Mutex::new(HashMap::new())),
            orders_blocked: Arc::new(Mutex::new(Vec::new())),
            condv_orders: Arc::new(Condvar::new()),
            delivery_process: Vec::new(),
            bernoulli_dist: Bernoulli::new(PROBABILITY_OF_SUCCESS_DELIVERY).unwrap(),
        };

        //Se implemento así porque Store no impmlementa el metodo clone.
        for i in 0..AMAOUNT_OF_DELIVERY_PROCESS {
            let products: Arc<Mutex<HashMap<i32, Product>>> = store.products.clone();
            let orders_blocked: Arc<Mutex<Vec<Product>>> = store.orders_blocked.clone();
            let condv_orders: Arc<Condvar> = store.condv_orders.clone();
            store.delivery_process.push(thread::spawn(move || {
                delivery_logic(
                    i,
                    products,
                    orders_blocked,
                    condv_orders,
                    store.bernoulli_dist,
                )
            }));
        }

        //Insertamos algunos productos dentro de mi stock. Es mi stock inicial.
        {
            let mut products_guard = store.products.lock().unwrap();
            for product_id in 0..10{
                let random_amount: i32 = thread_rng().gen_range(5..15);
                products_guard.insert(product_id, Product { id: product_id, amount: random_amount });
            }
        }
        store.bernoulli_dist = Bernoulli::new(PROBABILITY_OF_SUCCESS_DELIVERY)
            .expect("Error al crear la distribucion de Bernoulli");
        store
    }

    //Me devuelve si el producto esta disponible. En el caso de que este lo elimino del stock.
    pub fn get_product(&mut self, id: i32, amount: i32) -> bool {
        let mut products_guard = self.products.lock().unwrap();
        if let Some(product) = products_guard.get(&id) {
            println!("\x1b[34m[ACTOR STORE] Se encontro el producto\x1b[0m");
            if product.amount >= amount {
                let new_amount = product.amount - amount;
                products_guard.insert(
                    id,
                    Product {
                        id,
                        amount: new_amount,
                    },
                );
                println!("\x1b[32m[ACTOR STORE] Producto disponible para entregar\x1b[0m \n");
                true
            } else {
                println!("\x1b[31m[ACTOR STORE] No hay la cantidad requerida\x1b[0m \n");
                false
            }
        } else {
            println!("\x1b[31m[ACTOR STORE] No se encontro el producto\x1b[0m \n");
            false
        }
    }
    
    /*
    pub fn wait_for_delivery_completion(&mut self) {
        // Iterar sobre los threads de entrega sin mover el vector completo
        while let Some(handle) = self.delivery_process.pop() {
            handle.join().unwrap();
        }
    }
    */
}

impl Actor for Store {
    type Context = Context<Self>;
}

impl Handler<ReceiveOrder> for Store {
    type Result = bool;

    fn handle(&mut self, msg: ReceiveOrder, _ctx: &mut Self::Context) -> Self::Result {
        let id = msg.id;
        let amount = msg.amount;
        println!(
            "\x1b[34m[ACTOR STORE] Recibi un pedido de {} con una cantidad {}\x1b[0m",
            id, amount
        );
        //Busco si tengo stock
        self.get_product(id, amount)
    }
}

// Me llega un pedido de ecomerce lo bloqueo y lo mando a delivery
impl Handler<BlockProduct> for Store {
    type Result = ();

    fn handle(&mut self, msg: BlockProduct, _ctx: &mut Self::Context) -> Self::Result {
        self.orders_blocked.lock().unwrap().push(Product {
            id: msg.id,
            amount: msg.amount,
        });
        println!("\x1b[33m[ACTOR STORE] Producto bloqueado\x1b[0m");
        self.condv_orders.notify_all();
    }
}

fn delivery_logic(
    i: u32,
    products: Arc<Mutex<HashMap<i32, Product>>>,
    orders_blocked: Arc<Mutex<Vec<Product>>>,
    condv_orders: Arc<Condvar>,
    bernoulli_dist: Bernoulli,
) {
    loop {
        let product_to_deliver;
        {
            //Espero hasta que tenga algun producto para realizar el delivery
            let mut _guard = condv_orders
                .wait_while(orders_blocked.lock().unwrap(), |orders| {
                    // Espero mientras no haya productos para ser entregados
                    orders.is_empty()
                })
                .unwrap();
            // Saco un producto de la lista de ordenes.
            product_to_deliver = _guard.pop().unwrap();
        }
        println!(
            "\x1b[33m[DELIVERY {}] Comenzamos el delivery del producto\x1b[0m",
            i
        );
        thread::sleep(Duration::from_secs(thread_rng().gen_range(5..10) as u64));
        // Decidir si se resuelve el envio  o no
        let delivery_success = bernoulli_dist.sample(&mut thread_rng());
        if delivery_success {
            // Si se entrega correctamente el delivery
            println!(
                "\x1b[33m[DELIVERY {}] Se pudo entregar correctamente el producto {}\x1b[0m",
                i, product_to_deliver.id
            );
        } else {
            // En el caso de que no se pudo entregar el producto lo devuelvo al stock
            println!(
                "\x1b[33m[DELIVERY {}] No se pudo entregar el pedido {}\x1b[0m",
                i, product_to_deliver.id
            );
            if let Some(product_to_restore) =
                products.lock().unwrap().get_mut(&product_to_deliver.id)
            {
                product_to_restore.amount += product_to_deliver.amount;
            }
        }
    }
}
