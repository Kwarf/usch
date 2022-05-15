use crate::{*, time::TimeSource};

pub struct DemoBuilder {
    pub(super) demo: Demo,
}

impl DemoBuilder {
    pub fn new((width, height): (u32, u32), title: &'static str) -> DemoBuilder {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_inner_size(PhysicalSize { width, height })
            .with_title(title)
            .build(&event_loop)
            .unwrap();

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

        #[cfg(feature = "ui")]
        let ui = ui::Ui::new(&window, &device, surface.get_preferred_format(&adapter).unwrap());

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
                #[cfg(feature = "ui")]
                tracker: None,
                #[cfg(feature = "ui")]
                ui: ui,
            },
        }
    }

    pub fn scene(mut self, builder: impl Fn(SceneBuilder) -> Scene) -> DemoBuilder {
        self.demo.scenes.push(builder(SceneBuilder {
            demo_builder: &self,
            fragment_source: None,
            #[cfg(feature = "hot-reload")]
            fragment_source_watcher: None,
            uniforms: Box::new(|_| vec![]),
        }));
        self
    }

    pub fn build(self) -> Demo {
        self.demo
    }
}

pub struct SceneBuilder<'a> {
    demo_builder: &'a DemoBuilder,
    fragment_source: Option<&'static str>,
    #[cfg(feature = "hot-reload")]
    fragment_source_watcher: Option<SourceWatcher>,
    uniforms: Box<dyn Fn(&dyn TimeSource) -> Vec<u8>>,
}

impl<'a> SceneBuilder<'a> {
    pub fn with_uniforms(mut self, uniforms: impl Fn(&dyn TimeSource) -> Vec<u8> + 'static) -> SceneBuilder<'a> {
        self.uniforms = Box::new(uniforms);
        self
    }

    pub fn set_fragment_source(mut self, src: &'static str) -> SceneBuilder<'a> {
        self.fragment_source = Some(src);
        self
    }

    #[cfg(feature = "hot-reload")]
    pub fn watch_fragment_source(mut self, path: &std::path::Path) -> SceneBuilder<'a> {
        self.fragment_source_watcher = Some(SourceWatcher::new(path));
        self
    }

    pub fn build(self) -> Scene {
        let demo = &self.demo_builder.demo;

        let frag = wgpu::ShaderSource::SpirV(Cow::Owned(glsl::compile_fragment(
            self.fragment_source
                .expect("No fragment shader source provided"),
        )));

        Scene {
            pipeline: raymarching::build_pipeline(
                &demo.device,
                demo.get_preferred_format(),
                frag,
                &(self.uniforms)(&demo.time),
            ),
            #[cfg(feature = "hot-reload")]
            fragment_source_watcher: self.fragment_source_watcher,
            uniforms: self.uniforms,
        }
    }
}
