use std::path::Path;
use std::net::TcpStream;
use std::io::Read;

use ssh2::Session;


fn exec_ssh_command_once(command : String) -> Result<i32,Box<dyn std::error::Error>> {
	let tcp = TcpStream::connect("192.168.178.53:22")?;
	let mut sess = Session::new()?;
	sess.set_tcp_stream(tcp);
	sess.handshake()?;
	sess.userauth_pubkey_file("olimex", Some(Path::new("/home/olimex/.ssh/id_rsa.pub")), Path::new("/home/olimex/.ssh/id_rsa"), None)?;
	let mut channel = sess.channel_session()?;
	channel.exec(&command)?;
	let mut s = String::new();
	channel.read_to_string(&mut s)?;
	channel.wait_close().ok();
	return Ok(channel.exit_status()?);
}

fn exec_ssh_command_log_error(command : String) {
	let result = exec_ssh_command_once(command.to_string());
	match result {
		Ok(response) => {
			if response != 0 {
				eprintln!("Couldn't exec {} with exit status {}", &command, response);
			}
		},
		Err(error) => {
			eprintln!("Couldn't exec {} because {}", &command, error);
		},
	}
}

pub fn exec_ssh_command(command : String) {
	let result = exec_ssh_command_once(command.clone());
	match result {
		Ok(response) => {
			if response != 0 {
				exec_ssh_command_log_error(command.clone());
			}
		},
		Err(_error) => exec_ssh_command_log_error(command.clone()),
	}
}
