use futures::future::join_all;
use systemstat::Duration;
use tokio::{
	io::{self, AsyncBufReadExt, AsyncWriteExt},
	net::{TcpListener, TcpStream},
	sync::mpsc::{self, Receiver, Sender},
	time::interval,
};

async fn wait_for_news_sockets(sender: Sender<String>) {
	let news = TcpListener::bind("127.0.0.1:8080").await.unwrap();

	loop {
		let (socket, _) = news.accept().await.unwrap();
		tokio::spawn(handle_news_socket(socket, sender.clone()));
	}
}

async fn handle_news_socket(socket: TcpStream, sender: Sender<String>) {
	let mut stream = tokio::io::BufStream::new(socket);
	loop {
		let mut line = String::new();
		match stream.read_line(&mut line).await {
			Ok(0) => {
				// Connection closed
				break;
			}
			Ok(_) => {
				stream
					.write_all(b"You are now connected to news!")
					.await
					.unwrap();
				sender.send(line.to_owned()).await.unwrap()
			}
			Err(e) => {
				eprintln!("Error reading from socket: {}", e);
				break;
			}
		}
	}
}

async fn handle_subscriber(socket: TcpStream, mut receiver: Receiver<String>) {
	let mut writer = tokio::io::BufWriter::new(socket);
	while let Some(message) = receiver.recv().await {
		writer.write_all(message.as_bytes()).await.unwrap();
		writer.flush().await.unwrap();
	}
}

async fn annoying(sender: Sender<String>) {
	let mut interval = interval(Duration::from_secs(1));
	loop {
		sender.send(String::from("annoying")).await.unwrap();
		interval.tick().await;
	}
}

#[tokio::main]
async fn main() -> io::Result<()> {
	let mut tasks = vec![];

	let (sender, receiver) = mpsc::channel::<String>(32);
	tasks.push(tokio::spawn(wait_for_news_sockets(sender.clone())));

	let subscriber = TcpListener::bind("127.0.0.1:8090").await.unwrap();
	let (socket, _) = subscriber.accept().await.unwrap();
	tasks.push(tokio::spawn(handle_subscriber(socket, receiver)));
	tasks.push(tokio::spawn(annoying(sender.clone())));

	join_all(tasks).await;

	Ok(())
}

//
