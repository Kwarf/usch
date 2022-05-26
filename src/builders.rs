use std::path::PathBuf;

use winit::window::Fullscreen;

use crate::{time::TimeSource, *};

pub struct DemoBuilder {
    pub(super) demo: Demo,
}

impl DemoBuilder {
    pub fn new((width, height): (u32, u32), fullscreen: bool, title: &'static str) -> DemoBuilder {
        let event_loop = EventLoop::new();
        let resolution = PhysicalSize { width, height };
        let window = WindowBuilder::new()
            .with_inner_size(resolution)
            .with_fullscreen(DemoBuilder::fullscreen_mode(
                &event_loop,
                resolution,
                fullscreen,
            ))
            .with_title(title)
            .build(&event_loop)
            .unwrap();

        if fullscreen {
            window.set_cursor_visible(false)
        }

        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
        let surface = unsafe { instance.create_surface(&window) };

        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .unwrap();

        let (device, queue) = block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None,
        ))
        .unwrap();

        #[cfg(feature = "editor")]
        let ui = ui::Ui::new(
            &window,
            &device,
            surface.get_preferred_format(&adapter).unwrap(),
        );

        DemoBuilder {
            demo: Demo {
                event_loop,
                window,
                device,
                queue,
                surface,
                adapter,
                music: None,
                scenes: vec![],
                time: SeekableTimeSource::now(),
                #[cfg(feature = "editor")]
                tracker: None,
                #[cfg(feature = "editor")]
                ui: ui,
            },
        }
    }

    pub fn scene(mut self, builder: impl Fn(SceneBuilder) -> Scene) -> DemoBuilder {
        self.demo.scenes.push(builder(SceneBuilder {
            demo_builder: &self,
            #[cfg(not(feature = "editor"))]
            fragment_wgsl: None,
            #[cfg(feature = "editor")]
            fragment_glsl: None,
            #[cfg(feature = "editor")]
            fragment_source_watcher: None,
            #[cfg(feature = "editor")]
            glsl_include_paths: None,
            uniforms: Box::new(|_| vec![]),
        }));
        self
    }

    pub fn build(self) -> Demo {
        self.demo
    }

    fn fullscreen_mode(
        event_loop: &EventLoop<()>,
        resolution: PhysicalSize<u32>,
        fullscreen: bool,
    ) -> Option<winit::window::Fullscreen> {
        if !fullscreen {
            return None;
        }

        let video_mode = event_loop.primary_monitor()
            .unwrap()
            .video_modes()
            .find(|x| x.refresh_rate() == 60 && x.size() == resolution)
            .expect(&format!("Could not find a {}x{} @ 60Hz fullscreen video mode", resolution.width, resolution.height));

        Some(winit::window::Fullscreen::Exclusive(video_mode))
    }
}

pub struct SceneBuilder<'a> {
    demo_builder: &'a DemoBuilder,
    #[cfg(not(feature = "editor"))]
    fragment_wgsl: Option<&'static str>,
    #[cfg(feature = "editor")]
    fragment_glsl: Option<&'static str>,
    #[cfg(feature = "editor")]
    fragment_source_watcher: Option<SourceWatcher>,
    #[cfg(feature = "editor")]
    glsl_include_paths: Option<Vec<PathBuf>>,
    uniforms: Box<dyn Fn(&dyn TimeSource) -> Vec<u8>>,
}

impl<'a> SceneBuilder<'a> {
    pub fn with_uniforms(
        mut self,
        uniforms: impl Fn(&dyn TimeSource) -> Vec<u8> + 'static,
    ) -> SceneBuilder<'a> {
        self.uniforms = Box::new(uniforms);
        self
    }

    #[cfg(feature = "editor")]
    pub fn add_glsl_include_path(mut self, path: impl Into<PathBuf>) -> SceneBuilder<'a> {
        if self.glsl_include_paths.is_none() {
            self.glsl_include_paths = Some(vec![path.into()]);
        } else {
            self.glsl_include_paths.as_mut().unwrap().push(path.into());
        }
        self
    }

    #[cfg(feature = "editor")]
    pub fn set_fragment_source(mut self, src: &'static str) -> SceneBuilder<'a> {
        self.fragment_glsl = Some(src);
        self
    }

    #[cfg(not(feature = "editor"))]
    pub fn set_fragment_source(mut self, src: &'static str) -> SceneBuilder<'a> {
        self.fragment_wgsl = Some(src);
        self
    }

    #[cfg(feature = "editor")]
    pub fn watch_fragment_source(mut self, path: &std::path::Path) -> SceneBuilder<'a> {
        self.fragment_source_watcher = Some(SourceWatcher::new(path));
        self
    }

    pub fn build(self) -> Scene {
        let demo = &self.demo_builder.demo;

        #[cfg(feature = "editor")]
        let frag = wgpu::ShaderSource::SpirV(Cow::Owned(
            glsl::compile_fragment(
                self.fragment_glsl
                    .expect("No fragment shader source provided"),
                &self.glsl_include_paths,
            )
            .unwrap(),
        ));

        #[cfg(not(feature = "editor"))]
        let frag = wgpu::ShaderSource::Wgsl(Cow::Owned(self.fragment_wgsl.unwrap().to_string()));

        Scene {
            pipeline: raymarching::build_pipeline(
                &demo.device,
                demo.get_preferred_format(),
                frag,
                &(self.uniforms)(&demo.time),
            ),
            #[cfg(feature = "editor")]
            fragment_source_watcher: self.fragment_source_watcher,
            #[cfg(feature = "editor")]
            glsl_include_paths: self.glsl_include_paths,
            uniforms: self.uniforms,
        }
    }
}
