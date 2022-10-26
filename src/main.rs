use std::fs::{File, OpenOptions};
use anyhow::{anyhow, Error, Result};
use flatland::Flatland;
use input_window::InputWindow;
use manifest_dir_macros::directory_relative_path;
use stardust_xr_molecules::fusion::client::Client;
use std::thread;
use directories::ProjectDirs;
use tokio::{runtime::Handle, sync::oneshot};
use winit::{event_loop::EventLoopBuilder, platform::unix::EventLoopBuilderExtUnix};

pub mod cursor;
pub mod flatland;
pub mod input_window;
pub mod panel_ui;
// pub mod resize_handle;
pub mod util;
pub mod key_shortcuts;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
	let (client, stardust_event_loop) = Client::connect_with_async_loop().await?;
	client.set_base_prefixes(&[directory_relative_path!("res")]);

	let tokio_handle = Handle::current();
	let flatland = client.wrap_root(Flatland::new(client.clone()).await?);
	let (winit_stop_tx, mut winit_stop_rx) = oneshot::channel::<()>();
	let winit_thread = thread::Builder::new().name("winit".to_owned()).spawn({
		let flatland = flatland.weak_wrapped().upgrade().unwrap();
		move || -> Result<()> {
			let _tokio_guard = tokio_handle.enter();
			let event_loop = EventLoopBuilder::new()
				.with_any_thread(true)
				.with_x11()
				.build();
			let mut input_window = InputWindow::new(&event_loop, flatland)?;

			event_loop.run(move |event, _, control_flow| {
				match winit_stop_rx.try_recv() {
					Ok(_) => {
						control_flow.set_exit();
						return;
					}
					Err(ref e) if *e == oneshot::error::TryRecvError::Closed => {
						return;
					}
					_ => (),
				}

				input_window.handle_event(event);
			});
		}
	})?;

	let result = stardust_event_loop
		.await
		.map_err(|_| anyhow!("Server disconnected"));

	winit_stop_tx
		.send(())
		.expect("Failed to send stop signal to winit thread");
	winit_thread.join().expect("Couldn't rejoin winit thread")?;

	result
}

pub fn create_or_get_config_file() -> Result<File> {
	let project_dirs = ProjectDirs::from("", "", "flatland").expect("Error trying to get config directory");
	if !project_dirs.config_dir().exists() {
		std::fs::create_dir(project_dirs.config_dir())?;
		if !project_dirs.config_dir().join("config.kdl").exists() {
			let file = OpenOptions::new().read(true).write(true).create(true).open(project_dirs.config_dir().join("config.kdl"))?;
			return Ok(file);
		}
	}
	Err(Error::msg("unable to create config file"))
}