# Introduzione capolavoro
Essendo un amante del linguaggio di programmazione rust e affascinato dalla programmazione con correnti, ho deciso di implementare un web server molto basilare capace di rispondere a più richieste contemporaneamente, ma anche configurabile tramite un file toml.

Questa pagina è sia disponibile sul web server stesso a quest'[indirizzo](http://webserver.rust.mattwax.yxz) (ma sarà attivo solo per occasioni particolari), oppure vi si può accedere da questo link: [mattwax.xyz/rs/webserver/](https://mattwax.xyz/rs/webserver/).

# Il codice
## Il main
Partirei con il far vedere il `main` del programma:
```rust
use std::{
    fs,
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
    sync::Arc,
};
use webserver::{Config, Resource, Server, ThreadPool};

fn main() {
    let config = Config::from_default_config_file().unwrap();

    let Server {
        host,
        port,
        threads,
    } = config.server.as_ref().unwrap();

    let host = if let Some(host) = host {
        host
    } else {
        "127.0.0.1"
    };
    let port = if let Some(port) = port { port } else { "80" };
    let threads = if let Some(threads) = threads {
        threads
    } else {
        &1
    };

    let listener = TcpListener::bind(format!("{host}:{port}")).unwrap();
    let pool = ThreadPool::new(*threads);

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(stream) => stream,
            Err(_) => continue,
        };

        let config = Arc::clone(&config);

        pool.execute(|| {
            handle_connection(stream, config);
            println!("new request handled");
        });
    }
}
```

Come si può vedere il main è di dimensioni ridotte ed è incaricato soltanto di creare un ascoltatore sulla porta definita nel file di configurazione e smistare le richieste alla funzione `handle_connection()`.
Adesso scendiamo più nel dettaglio di quello che succede dietro le quinte.

### Variabili dichiarate
Per prima cosa raccogliamo le informazioni dal file di configurazione nella location di default, che per scopi di debugging è 
```rust
let config = Config::from_default_config_file().unwrap();
```






```rust
let Server {
    host,
    port,
    threads,
} = config.server.as_ref().unwrap();

let host = if let Some(host) = host {
    host
} else {
    "127.0.0.1"
};
let port = if let Some(port) = port { port } else { "80" };
let threads = if let Some(threads) = threads {
    threads
} else {
    &1
};

let listener = TcpListener::bind(format!("{host}:{port}")).unwrap();
let pool = ThreadPool::new(*threads);
```
