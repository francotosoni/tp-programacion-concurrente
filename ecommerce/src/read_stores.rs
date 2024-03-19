use csv::ReaderBuilder;
use std::collections::HashMap;
use std::error::Error;

// Lee un archivo CSV y crea un mapa de tiendas.
//
// Esta función lee un archivo CSV cuya ruta se especifica en `file_path`.
// Cada línea del archivo CSV se espera que contenga un identificador de tienda y una dirección IP,
// separados por una coma. La función crea y devuelve un mapa donde cada identificador
// se asocia con su correspondiente dirección IP.
//
// Argumentos:
// * `file_path`: Una referencia a una cadena de texto que representa la ruta del archivo CSV a leer.
//
// Retorna:
// Un `Result` que contiene un `HashMap<String, String>` si la lectura es exitosa.
// Cada clave del `HashMap` es el identificador de una tienda y su valor es la dirección IP correspondiente.
// En caso de error en la lectura del archivo o en el procesamiento de los datos, retorna un `Error`.
pub fn read_stores(file_path: &str) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new().from_path(file_path)?;
    let mut stores = HashMap::new();

    for result in rdr.records() {
        let record = result?;
        let id = record.get(0).unwrap().to_string();
        let ip = record.get(1).unwrap().to_string();
        stores.insert(id, ip);
    }

    Ok(stores)
}
