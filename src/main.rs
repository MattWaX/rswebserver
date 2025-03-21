use std::{
    fs,
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
    sync::Arc,
};
use webserver::{Config, Server, ThreadPool};

fn main() {
    let config = Config::from_default_config_file().unwrap();
    dbg!(&config);
    //let host = &config.server.as_ref().unwrap().host.as_ref().unwrap();
    //let port = &config.server.as_ref().unwrap().port.as_ref().unwrap();
    let Server {
        host,
        port,
        threads,
    } = config.server.as_ref().unwrap();
    let host = if let Some(host) = host { host } else { "127.0.0.1" };
    let port = if let Some(port) = port { port } else { "80" };
    let threads = if let Some(threads) = threads { threads } else { &1 };
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

fn handle_connection(mut stream: TcpStream, config: Arc<Config>) {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let (status_line, filename) = if request_line == "GET / HTTP/1.1" {
        ("HTTP/1.1 200 OK\r\n\r\n", "test.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();
    let response = format!("{status_line}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}
