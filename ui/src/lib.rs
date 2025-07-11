use std::marker::PhantomData;

use app_base::{
    Plugin,
    dpi::PhysicalSize,
    math::Vec4,
    storage::{Res, ResMut, Resource},
    system::scheduler::{PostUpdate, PreUpdate, Update},
    window::WindowId,
};

use clay_layout::render_commands::RenderCommand;
pub use clay_layout::*;
use log::warn;
use renderer::{Mesh, Meshes, RenderResources};
use window::Windows;

#[derive(Debug, Default)]
pub struct UiPlugin<T: UiBuilder + Resource> {
    _pd: PhantomData<T>,
}

pub struct UiCommands<'a, ImageElementData, CustomElementData> {
    render_commands: Vec<RenderCommand<'a, ImageElementData, CustomElementData>>,
    window: WindowId,
}

pub trait UiBuilder {
    type ImageElementData;
    type CustomElementData;
    fn build<'a>(
        &mut self,
        window_size: PhysicalSize<u32>,
    ) -> Vec<RenderCommand<'a, Self::ImageElementData, Self::CustomElementData>>;
    fn for_window(&self) -> WindowId;
}

impl<T: UiBuilder + Resource + 'static> Plugin for UiPlugin<T> {
    fn build(&self, app: &mut app_base::App) {
        app.add_systems(Update, prepare_layout::<T>);
        app.add_systems(PostUpdate, render_layout::<T>);
    }
}

fn prepare_layout<T: UiBuilder + Resource + 'static>(
    mut builder: ResMut<T>,
    windows: Res<Windows>,
    mut meshes: ResMut<Meshes>,
) {
    let window = builder.for_window();
    if let Some(window) = windows.get_window(window) {
        let window_size = window.inner_size();
        let commands = builder.build(window_size);
        for c in commands {
            match c.config {
                render_commands::RenderCommandConfig::None() => {}
                render_commands::RenderCommandConfig::Rectangle(rectangle) => {
                    let bb = c.bounding_box;
                    let mesh = Mesh {
                        vertices: vec![
                            Vec4::new(bb.x, bb.y, 0.0, 1.0),
                            Vec4::new(bb.x + bb.width, bb.y, 0.0, 1.0),
                            Vec4::new(bb.x + bb.width, bb.y + bb.height, 0.0, 1.0),
                            Vec4::new(bb.x, bb.y + bb.height, 0.0, 1.0),
                        ],
                        indices: vec![0, 1, 2, 2, 3, 0],
                    };
                    let id = meshes.add_mesh(mesh);
                }
                render_commands::RenderCommandConfig::Border(border) => todo!(),
                render_commands::RenderCommandConfig::Text(text) => todo!(),
                render_commands::RenderCommandConfig::Image(image) => todo!(),
                render_commands::RenderCommandConfig::ScissorStart() => todo!(),
                render_commands::RenderCommandConfig::ScissorEnd() => todo!(),
                render_commands::RenderCommandConfig::Custom(custom) => todo!(),
            }
        }
    } else {
        warn!(
            "Window not found: {window:?}",
            window = builder.for_window()
        );
    }
}

fn render_layout<T: UiBuilder + Resource + 'static>(
    mut builder: ResMut<T>,
    render_resources: Res<RenderResources>,
) {
    let window = builder.for_window();
    if let Some(render_resources) = render_resources.get_resource(&window) {
        let commands = builder.build(PhysicalSize {
            width: render_resources.surface_config.width,
            height: render_resources.surface_config.height,
        });

        for c in commands {
            match c.config {
                render_commands::RenderCommandConfig::None() => {}
                render_commands::RenderCommandConfig::Rectangle(rectangle) => {}
                render_commands::RenderCommandConfig::Border(border) => todo!(),
                render_commands::RenderCommandConfig::Text(text) => todo!(),
                render_commands::RenderCommandConfig::Image(image) => todo!(),
                render_commands::RenderCommandConfig::ScissorStart() => todo!(),
                render_commands::RenderCommandConfig::ScissorEnd() => todo!(),
                render_commands::RenderCommandConfig::Custom(custom) => todo!(),
            }
        }
    } else {
        warn!("no render resources found for window {:?}", window);
    }
}
