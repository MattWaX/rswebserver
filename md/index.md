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

Molto semplicemente quello che fa questa funzione è ottenere lo stream per poterci leggere e scrivere sopra, ottiene la richiesta e la salviamo in `request_line`, una volta ottenuta controlliamo con il costrutto `match` (analogo allo `switch` in altri linguaggi, ma molto più espressivo) se la pagina esiste tra quelle elencate nel file di configurazione, in caso contrario il caso di default `_` porterà alla pagina `404.html` per segnalare che il contenuto non esiste.
Adesso possiamo leggere il contenuto della pagina richiesta e scrivere sullo stream la risposta http.
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
## Contenuto di lib.rs
Oltre all'implementazione della deserializzazione del file di configurazione, all'interno del file `lib.rs` vi si trova il codice per la gestione della `ThreadPool` e dei `Worker`.

### Worker
La struct che definisce i `Worker` è molto semplice:
```rust
pub struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}
```
Abbiamo infatti solo un id per poter differenziare i `Worker` e l'handle che gli permette di attaccarsi ad i thread in arrivo.

Implementiamo poi il metodo per poter creare un nuovo `Worker` che rimarrà in ascolto di nuovi `job` su un suo thread apposito da eseguire.
Restituiamo alla fine `Worker { id, thread }` per essere gestito dalla `ThreadPool`.
```rust
impl Worker {
    /// Create a new worker that listen for new jobs to execute
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let message = receiver.lock().unwrap().recv();

                match message {
                    Ok(job) => {
                        println!("Worker {id} got a job; executing.");
                        job();
                    }
                    Err(_) => {
                        println!("Worker {id} disconnected! Shutting down!");
                        break;
                    }
                }
            }
        });

        Worker { id, thread }
    }
}
```

`job` è un abbreviazione il tipo completo è definito cosi: 
```rust
type Job = Box<dyn FnOnce() + Send + 'static>;
```
Ovvero un puntatore `Box<T>` ad una funzione col tratto `Send`.

### ThreadPool
Nella struct che definisce la `ThreadPool` abbiamo come attributi un vettore di `Worker` e il canale per mandare i thread che riceviamo ai nostri `Worker`.
```rust
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}
```

Le due funzioni implementate servono rispettivamente per inizializzare la `ThreadPool` e per mandare richieste ai `Worker` liberi.
```rust
impl ThreadPool {
    /// Create a ThreadPool
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for i in 0..size {
            workers.push(Worker::new(i, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    /// Send the job to a worker in the `ThreadPool`
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}
```

Non dobbiamo scordarci di implementare anche il tratto `Drop` dove nella funzione `drop(&mut self)` andiamo a descrivere come si dovrà comportare il programma quando la `ThreadPool` cadrà fuori dallo scopo e quindi dovrà essere liberata dalla memoria.
```rust
impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers.drain(..) {
            println!("Shutting down worker {}", worker.id);

            worker.thread.join().unwrap();
        }
    }
}
```

# Conclusione
Rust è un linguaggio molto complesso, ma anche estremamente espressivo, dove il focus principale è garantire l'utilizzo della memoria in modo sicuro e altre velocità di esecuzione, ciò lo rende un linguaggio molto adatto a questo genere di mansioni ed anche divertente da scrivere.

Grazie di aver letto fino ad ora spero possa essere stato interessante.

</main>
