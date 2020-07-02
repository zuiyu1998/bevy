use crate::app::AppBuilder;

pub trait AddDefaultPlugins {
    fn add_default_plugins(&mut self) -> &mut Self;
}

impl AddDefaultPlugins for AppBuilder {
    fn add_default_plugins(&mut self) -> &mut Self {
        self.add_plugin(crate::type_registry::TypeRegistryPlugin::default());
        self.add_plugin(crate::core::CorePlugin::default());
        self.add_plugin(crate::diagnostic::DiagnosticsPlugin::default());
        self.add_plugin(crate::input::InputPlugin::default());
        self.add_plugin(crate::window::WindowPlugin::default());
        self.add_plugin(crate::asset::AssetPlugin::default());
        self.add_plugin(crate::scene::ScenePlugin::default());
        self.add_plugin(crate::render::RenderPlugin::default());
        self.add_plugin(crate::sprite::SpritePlugin::default());
        self.add_plugin(crate::pbr::PbrPlugin::default());
        self.add_plugin(crate::ui::UiPlugin::default());
        self.add_plugin(crate::gltf::GltfPlugin::default());
        self.add_plugin(crate::text::TextPlugin::default());

        #[cfg(feature = "bevy_winit")]
        self.add_plugin(winit::WinitPlugin::default());
        #[cfg(not(feature = "bevy_winit"))]
        self.add_plugin(crate::app::schedule_runner::ScheduleRunnerPlugin::default());

        #[cfg(feature = "bevy_wgpu")]
        self.add_plugin(wgpu::WgpuPlugin::default());

        self
    }
}
