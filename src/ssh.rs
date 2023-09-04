use tokio::net::TcpStream;
use std::path::Path;
use async_ssh2_tokio::*;

const SSH_HOST_IP: &str = "192.168.178.53";

pub async fn exec_ssh_command(command: String) {
	let result = exec_ssh_command_once(command.clone()).await;
		if response != 0 {
			exec_ssh_command_log_error(command.clone());
		} else {
			exec_ssh_command_log_error(command.clone()).await;
		}
	}
}

async fn exec_ssh_command_once(command: String) -> i32 {
	let auth_method = AuthMethod::PrivateKeyFile { key_file_name: String::from("/home/olimex/.ssh/id_rsa"), key_pass: None };
	let mut client = Client::connect(
        (SSH_HOST_IP, 22),
        "olimex",
        auth_method,
        ServerCheckMethod::NoCheck,
    ).await?;
	let result = client.execute(&command).await;
	match result {
		Ok(r) => r.exit_status as i32,
		Err(e) => panic!("TODO"),
	}
}