use winit::{
    event::{ Event, KeyEvent, WindowEvent },
    event_loop::EventLoop,
    keyboard::{ Key, NamedKey },
};

mod renderer;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut renderer = pollster::block_on(renderer::Renderer::new(800, 600, &event_loop, "rem"));

    event_loop
        .run(move |event, target| {
            if let Event::WindowEvent { window_id: _, event } = event {
                match event {
                    WindowEvent::Resized(size) => {
                        renderer.resize(size.width, size.height);
                    }
                    WindowEvent::RedrawRequested => {
                        renderer.redraw();
                    }
                    WindowEvent::CloseRequested => target.exit(),
                    WindowEvent::KeyboardInput { event, .. } => {
                        match event {
                            KeyEvent { state, logical_key, .. } => {
                                if state == winit::event::ElementState::Pressed {
                                    match logical_key {
                                        Key::Character(c) => {
                                            // println!("Pressed character: {}", c);
                                            renderer.push_str(c.as_str());
                                            renderer.redraw();
                                        }
                                        Key::Named(c) => {
                                            // println!("Pressed named: {:?}", c);
                                            match c {
                                                NamedKey::Backspace => {
                                                    renderer.pop_char();
                                                }
                                                NamedKey::Escape => {
                                                    renderer.clear();
                                                }
                                                _ => {
                                                    renderer.push_str(c.to_text().unwrap_or(""));
                                                }
                                            }
                                            renderer.redraw();
                                        }
                                        Key::Unidentified(c) => {
                                            println!("Pressed unidentified: {:?}", c);
                                        }
                                        _ => {}
                                    }
                                }
                            }

                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        })
        .unwrap();
}
