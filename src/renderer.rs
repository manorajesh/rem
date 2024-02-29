use glyphon::{
    Attrs,
    Buffer,
    Color,
    Family,
    FontSystem,
    Metrics,
    Resolution,
    Shaping,
    SwashCache,
    TextArea,
    TextAtlas,
    TextBounds,
    TextRenderer,
};
use wgpu::{
    CommandEncoderDescriptor,
    CompositeAlphaMode,
    DeviceDescriptor,
    Features,
    Instance,
    InstanceDescriptor,
    Limits,
    LoadOp,
    MultisampleState,
    Operations,
    PresentMode,
    RenderPassColorAttachment,
    RenderPassDescriptor,
    RequestAdapterOptions,
    SurfaceConfiguration,
    TextureFormat,
    TextureUsages,
    TextureViewDescriptor,
};
use winit::{ dpi::LogicalSize, event_loop::EventLoop, window::WindowBuilder };

use std::sync::Arc;

pub struct Renderer<'a> {
    text_renderer: TextRenderer,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'a>,
    config: SurfaceConfiguration,
    atlas: TextAtlas,
    font_system: FontSystem,
    buffer: Buffer,
    cache: SwashCache,
    window: Arc<winit::window::Window>,
    pub text: String,
}

impl Renderer<'static> {
    pub async fn new(width: u32, height: u32, event_loop: &EventLoop<()>, title: &str) -> Self {
        let window = Arc::new(
            WindowBuilder::new()
                .with_inner_size(LogicalSize::new(width as f64, height as f64))
                .with_title(title)
                .build(event_loop)
                .unwrap()
        );
        let size = window.inner_size();
        let scale_factor = window.scale_factor();

        // Set up surface
        let instance = Instance::new(InstanceDescriptor::default());
        let adapter = instance.request_adapter(&RequestAdapterOptions::default()).await.unwrap();
        let (device, queue) = adapter
            .request_device(
                &(DeviceDescriptor {
                    label: None,
                    required_features: Features::empty(),
                    required_limits: Limits::default(),
                }),
                None
            ).await
            .unwrap();

        let surface = instance.create_surface(window.clone()).expect("Create surface");
        let swapchain_format = TextureFormat::Bgra8UnormSrgb;
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Immediate,
            alpha_mode: CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Set up text renderer
        let mut font_system = FontSystem::new();
        let cache = SwashCache::new();
        let mut atlas = TextAtlas::new(&device, &queue, swapchain_format);
        let text_renderer = TextRenderer::new(
            &mut atlas,
            &device,
            MultisampleState::default(),
            None
        );
        let mut buffer = Buffer::new(&mut font_system, Metrics::new(30.0, 42.0));

        let physical_width = ((width as f64) * scale_factor) as f32;
        let physical_height = ((height as f64) * scale_factor) as f32;

        buffer.set_size(&mut font_system, physical_width, physical_height);
        buffer.set_text(
            &mut font_system,
            "Hello world! 👋\nThis is rendered with 🦅 glyphon 🦁\nThe text below should be partially clipped.\na b c d e f g h i j k l m n o p q r s t u v w x y z",
            Attrs::new().family(Family::SansSerif),
            Shaping::Advanced
        );
        buffer.shape_until_scroll(&mut font_system);

        return Self {
            text_renderer,
            device,
            queue,
            surface,
            config,
            atlas,
            font_system,
            buffer,
            cache,
            window,
            text: String::new(),
        };
    }

    pub fn redraw(&mut self) {
        self.text_renderer
            .prepare(
                &self.device,
                &self.queue,
                &mut self.font_system,
                &mut self.atlas,
                Resolution {
                    width: self.config.width,
                    height: self.config.height,
                },
                [
                    TextArea {
                        buffer: &self.buffer,
                        left: 10.0,
                        top: 10.0,
                        scale: 1.0,
                        bounds: TextBounds {
                            left: 0,
                            top: 0,
                            right: self.config.width as i32,
                            bottom: self.config.height as i32,
                        },
                        default_color: Color::rgb(255, 255, 255),
                    },
                ],
                &mut self.cache
            )
            .unwrap();

        let frame = self.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(
            &(CommandEncoderDescriptor { label: None })
        );
        {
            let mut pass = encoder.begin_render_pass(
                &(RenderPassDescriptor {
                    label: None,
                    color_attachments: &[
                        Some(RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: Operations {
                                load: LoadOp::Clear(wgpu::Color::BLACK),
                                store: wgpu::StoreOp::Store,
                            },
                        }),
                    ],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                })
            );

            self.text_renderer.render(&self.atlas, &mut pass).unwrap();
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();

        self.atlas.trim();
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.window.request_redraw();
    }

    pub fn close(&self) {}

    // buffer manipluation
    pub fn refresh_buffer(&mut self) {
        self.buffer.set_text(
            &mut self.font_system,
            self.text.as_str(),
            Attrs::new().family(Family::SansSerif),
            Shaping::Advanced
        );
        self.buffer.shape_until_scroll(&mut self.font_system);
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.refresh_buffer();
    }

    pub fn push_str(&mut self, text: &str) {
        self.text.push_str(text);
        self.refresh_buffer();
    }

    pub fn pop_char(&mut self) {
        self.text.pop();
        self.refresh_buffer();
    }
}
