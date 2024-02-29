use std::os::fd::RawFd;

use winit::{
    event::{ Event, KeyEvent, WindowEvent },
    event_loop::EventLoop,
    keyboard::{ Key, NamedKey },
};

use nix::pty::forkpty;
use nix::unistd::ForkResult;
use nix::unistd::read;
use std::process::Command;

mod renderer;

fn read_from_fd(fd: RawFd) -> Option<Vec<u8>> {
    // https://linux.die.net/man/7/pipe
    let mut read_buffer = [0; 65536];
    let read_result = read(fd, &mut read_buffer);
    match read_result {
        Ok(bytes_read) => Some(read_buffer[..bytes_read].to_vec()),
        Err(_e) => None,
    }
}

fn spawn_pty_with_shell(default_shell: String) -> RawFd {
    match forkpty(None, None) {
        Ok(fork_pty_res) => {
            let stdout_fd = fork_pty_res.master; // primary
            if let ForkResult::Child = fork_pty_res.fork_result {
                // I'm the secondary part of the pty
                Command::new(&default_shell).spawn().expect("failed to spawn");
                std::thread::sleep(std::time::Duration::from_millis(2000));
                std::process::exit(0);
            }
            stdout_fd
        }
        Err(e) => {
            panic!("failed to fork {:?}", e);
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut renderer = pollster::block_on(renderer::Renderer::new(800, 600, &event_loop, "rem"));

    let default_shell = std::env::var("SHELL").expect("could not find default shell from $SHELL");
    let stdout_fd = spawn_pty_with_shell(default_shell);

    let mut read_buffer = vec![];
    loop {
        match read_from_fd(stdout_fd) {
            Some(mut read_bytes) => {
                read_buffer.append(&mut read_bytes);
            }
            None => {
                println!("{:?}", String::from_utf8(read_buffer).unwrap());
                std::process::exit(0);
            }
        }
    }

    // event_loop
    //     .run(move |event, target| {
    //         if let Event::WindowEvent { window_id: _, event } = event {
    //             match event {
    //                 WindowEvent::Resized(size) => {
    //                     renderer.resize(size.width, size.height);
    //                 }
    //                 WindowEvent::RedrawRequested => {
    //                     renderer.redraw();
    //                 }
    //                 WindowEvent::CloseRequested => target.exit(),
    //                 WindowEvent::KeyboardInput { event, .. } => {
    //                     match event {
    //                         KeyEvent { state, logical_key, .. } => {
    //                             if state == winit::event::ElementState::Pressed {
    //                                 match logical_key {
    //                                     Key::Character(c) => {
    //                                         // println!("Pressed character: {}", c);
    //                                         renderer.push_str(c.as_str());
    //                                         renderer.redraw();
    //                                     }
    //                                     Key::Named(c) => {
    //                                         // println!("Pressed named: {:?}", c);
    //                                         match c {
    //                                             NamedKey::Backspace => {
    //                                                 renderer.pop_char();
    //                                             }
    //                                             NamedKey::Escape => {
    //                                                 renderer.clear();
    //                                             }
    //                                             _ => {
    //                                                 renderer.push_str(c.to_text().unwrap_or(""));
    //                                             }
    //                                         }
    //                                         renderer.redraw();
    //                                     }
    //                                     Key::Unidentified(c) => {
    //                                         println!("Pressed unidentified: {:?}", c);
    //                                     }
    //                                     _ => {}
    //                                 }
    //                             }
    //                         }

    //                         _ => {}
    //                     }
    //                 }
    //                 _ => {}
    //             }
    //         }
    //     })
    //     .unwrap();
}
