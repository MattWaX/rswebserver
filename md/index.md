<main>
# Introduzione capolavoro
Essendo un amante del linguaggio di programmazione rust e affascinato dalla programmazione con correnti, ho deciso di implementare un web server molto basilare capace di rispondere a più richieste contemporaneamente, ma anche configurabile tramite un file toml.

Questa pagina è sia disponibile sul web server stesso a quest'[indirizzo](http://webserver.rust.mattwax.yxz) (ma sarà attivo solo per occasioni particolari), oppure vi si può accedere da questo link: [mattwax.xyz/rs/webserver/](https://mattwax.xyz/rs/webserver/).
Il repository di git con il codice sorgente e questa pagina web è presente a quest'[indirizzo](https://github.com/MattWaX/rswebserver).

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

### Inizializzazione delle strutture di dati necessarie
Per prima cosa raccogliamo le informazioni dal file di configurazione nella location di default.
```rust
let config = Config::from_default_config_file().unwrap();
```
`Config` è definita in `src/lib.rs` nel seguente modo:
```rust
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    #[allow(dead_code)]
    pub server: Option<Server>,
    #[allow(dead_code)]
    pub resources: Option<Vec<Resource>>,
}

#[derive(Deserialize, Debug)]
pub struct Server {
    #[allow(dead_code)]
    pub host: Option<String>,
    #[allow(dead_code)]
    pub port: Option<String>,
    #[allow(dead_code)]
    pub threads: Option<usize>,
}

#[derive(Deserialize, Debug)]
pub struct Resource {
    #[allow(dead_code)]
    pub request: String,
    #[allow(dead_code)]
    pub response: String,
}

impl Config {
    pub fn from_default_config_file() -> Result<Arc<Config>, Box<dyn Error>> {
        //let etc_path = "/etc/rustweb/conf.toml"; 
        //^^^ è il vero percorso che si utilizzerebbe in un sistema unix
        let etc_path = "/home/MattWaX/exercise-code/rust/rswebserver/conf.toml";
        if fs::exists(etc_path)? {
            let toml_str = fs::read_to_string(Path::new(&etc_path))?;
            let config: Config = toml::from_str(&toml_str)?;

            Ok(Arc::new(config))
        } else {
            todo!("Create a new config file if not existant");
        }
    }
}
```
Per facilitare il parsing dei dati contenuti nel file di configurazione, ho utilizzato la crate `Serde` (simile ad una libreria nel uso pratico) e da essa la macro `Deserialize` che si occupa della deserializzazione del file nel metodo che legge il file di configurazione `from_default_config_file()`.
La struttura `Config` è composta da una struttura `Server` che poi al suo interno contiene le specifiche di come verrà avviato il server (host, porta e quanti Thread saranno nella Thread Pool) e da un vettore di `Resource` che conterrà tutte le pagine del sito e il loro url.

Successivamente nel `main` andiamo a scomporre il campo `Server` dalla struct `Config`. 
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
```
Adesso possiamo finalmente avviare il server su host e porta specificate in `conf.toml` e definire anche quanti `Worker` ci saranno nella `ThreadPool`, non vogliamo esporre il server cosi alla possibilità di avere un numero potenzialmente infinito di threads in esecuzione contemporaneamente e quindi essere più vulnerabili ad un attacco di tipo DoS o più probabilmente un sovraccarico per il troppo traffico.
```rust
let listener = TcpListener::bind(format!("{host}:{port}")).unwrap();
let pool = ThreadPool::new(*threads);
```

### Gestione delle richieste
La logica per gestire le richieste http è molto semplice, per ogni richesta che arriva nello stream controlliamo se ha un valore, nel caso sia vero cloniamo lo smart pointer `Arc` che punta a `&config` e passiamo la funzione che contiene le istruzioni per rispondere alla richiesta, `handle_connection()` alla `ThreadPool`.
```rust
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
```

<!-- TODO:handle -->
```rust
fn handle_connection(mut stream: TcpStream, config: Arc<Config>) {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let resources = match &config.resources {
        Some(resources) => resources,
        _ => &vec![Resource {
            request: "/".to_string(),
            response: "www/404.html".to_string(),
        }],
    };

    for resource in resources {
        if request_line == format!("GET {} HTTP/1.1", resource.request) {
            let contents = fs::read_to_string(&resource.response).unwrap();
            let response = format!("HTTP/1.1 200 OK\r\n\r\n{}", contents);
            stream.write_all(response.as_bytes()).unwrap();
            return;
        }
    }
}
```
`...`
</main>
