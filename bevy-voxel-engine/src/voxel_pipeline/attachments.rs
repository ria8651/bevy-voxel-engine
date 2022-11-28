use crate::VoxelCamera;
use bevy::{
    ecs::query::QueryItem,
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_resource::*,
    },
};

pub struct AttachmentsPlugin;

impl Plugin for AttachmentsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ExtractComponentPlugin::<RenderAttachments>::default())
            .add_system(add_render_attachments)
            .add_system(update_textures);
    }
}

#[derive(Component, Clone)]
pub struct RenderAttachments {
    current_size: UVec2,
    pub normal: Handle<Image>,
    pub position: Handle<Image>,
}

impl ExtractComponent for RenderAttachments {
    type Query = &'static Self;
    type Filter = ();

    fn extract_component(item: QueryItem<'_, Self::Query>) -> Self {
        item.clone()
    }
}

fn add_render_attachments(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut query: Query<Entity, (With<VoxelCamera>, Without<RenderAttachments>)>,
) {
    for entity in query.iter_mut() {
        let size = Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };

        let mut normal_image = Image::new_fill(
            size,
            TextureDimension::D2,
            &[0, 0, 0, 0],
            TextureFormat::Rgba8Unorm,
        );
        normal_image.texture_descriptor.usage = TextureUsages::COPY_DST
            | TextureUsages::STORAGE_BINDING
            | TextureUsages::TEXTURE_BINDING;

        let mut position_image = Image::new_fill(
            size,
            TextureDimension::D2,
            &[0, 0, 0, 0],
            TextureFormat::Rgba16Float,
        );
        position_image.texture_descriptor.usage = TextureUsages::COPY_DST
            | TextureUsages::STORAGE_BINDING
            | TextureUsages::TEXTURE_BINDING;

        commands.entity(entity).insert(RenderAttachments {
            current_size: UVec2::new(1, 1),
            normal: images.add(normal_image),
            position: images.add(position_image),
        });
    }
}

fn update_textures(
    mut images: ResMut<Assets<Image>>,
    windows: Res<Windows>,
    mut query: Query<&mut RenderAttachments>,
) {
    let window = windows.get_primary().unwrap();
    let size = UVec2::new(window.physical_width(), window.physical_height());

    for mut render_attachments in query.iter_mut() {
        if size != render_attachments.current_size {
            render_attachments.current_size = size;
            info!(
                "Resizing attachments to width: {}, height: {}",
                size.x, size.y
            );

            let size = Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            };

            let normal_image = images.get_mut(&render_attachments.normal).unwrap();
            normal_image.resize(size);

            let position_image = images.get_mut(&render_attachments.position).unwrap();
            position_image.resize(size);
        }
    }
}
