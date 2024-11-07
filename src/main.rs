use std::io::{self, Error, ErrorKind, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;

fn http_start<A, F>(addr: A, proc: F) -> io::Result<()>
where
    A: ToString,
    F: FnMut(Result<TcpStream, Error>) + Send + 'static + Clone,
{
    let mut error_occurs = false;
    let (send_err, recv_err) = mpsc::channel::<Error>();
    let server = TcpListener::bind(addr.to_string())?;

    for request in server.incoming() {
        let mut cloned_func = proc.clone();
        let cloned_sender = send_err.clone();
        let task = thread::spawn(move || cloned_func(request));

        task.join().unwrap_or_else(|error| {
            error_occurs = true;

            let error = format!("{error:?}");
            let sys_error = Error::new(ErrorKind::Other, error);

            cloned_sender
                .send(sys_error)
                .unwrap_or_else(|send_failed| eprintln!("{send_failed}"));
        });

        if error_occurs {
            let reciever = recv_err
                .recv()
                .unwrap_or_else(|error| Error::new(ErrorKind::Other, format!("{error}")));

            return Err(reciever);
        }
    }

    Ok(())
}

fn main() {
    let result = http_start("127.0.0.1:3000", |request| {
	println!("accepted request!");

        let mut stream = request.unwrap();
        let buffer = std::fs::read_to_string("./src/main.rs").unwrap();
        let _ = stream.write(buffer.as_bytes()).unwrap();
    });

    result.unwrap_or(());
}
