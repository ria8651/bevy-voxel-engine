use crate::TraceSettings;
use bevy::{
    ecs::query::QueryItem,
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_asset::RenderAssets,
        render_graph::{self, NodeRunError, RenderGraphContext, SlotInfo, SlotType, SlotValue},
        render_resource::*,
        renderer::RenderContext,
    },
};

pub struct AttachmentsPlugin;

impl Plugin for AttachmentsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ExtractComponentPlugin::<RenderAttachments>::default())
            .add_system(add_render_attachments)
            .add_system(update_attachments);
    }
}

#[derive(Component, Clone)]
pub struct RenderAttachments {
    current_size: UVec2,
    pub colour: Handle<Image>,
    pub last_colour: Handle<Image>,
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
    mut query: Query<Entity, (With<TraceSettings>, Without<RenderAttachments>)>,
) {
    for entity in query.iter_mut() {
        let size = Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };
        let mut image = Image::new_fill(
            size,
            TextureDimension::D2,
            &[0, 0, 0, 0],
            TextureFormat::Rgba16Float,
        );
        image.texture_descriptor.usage = TextureUsages::COPY_DST
            | TextureUsages::STORAGE_BINDING
            | TextureUsages::TEXTURE_BINDING;

        commands.entity(entity).insert(RenderAttachments {
            current_size: UVec2::new(1, 1),
            colour: images.add(image.clone()),
            last_colour: images.add(image.clone()),
            normal: images.add(image.clone()),
            position: images.add(image.clone()),
        });
    }
}

fn update_attachments(
    windows: Res<Windows>,
    mut images: ResMut<Assets<Image>>,
    mut query: Query<(&mut RenderAttachments, &Camera)>,
) {
    for (i, (mut render_attachments, camera)) in query.iter_mut().enumerate() {
        let render_target_info = camera.target.get_render_target_info(&windows, &images);
        let size = render_target_info.unwrap().physical_size;

        if size != render_attachments.current_size {
            render_attachments.current_size = size;
            info!(
                "Resizing camera {}s attachments to ({}, {})",
                i, size.x, size.y
            );

            let size = Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            };

            let colour_image = images.get_mut(&render_attachments.colour).unwrap();
            colour_image.resize(size);

            let last_colour_image = images.get_mut(&render_attachments.last_colour).unwrap();
            last_colour_image.resize(size);

            let normal_image = images.get_mut(&render_attachments.normal).unwrap();
            normal_image.resize(size);

            let position_image = images.get_mut(&render_attachments.position).unwrap();
            position_image.resize(size);
        }
    }
}

pub struct AttachmentsNode {
    query: QueryState<&'static RenderAttachments>,
}

impl AttachmentsNode {
    pub fn new(world: &mut World) -> Self {
        Self {
            query: world.query_filtered(),
        }
    }
}

impl render_graph::Node for AttachmentsNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![SlotInfo::new("view", SlotType::Entity)]
    }

    fn output(&self) -> Vec<SlotInfo> {
        vec![
            SlotInfo::new("colour", SlotType::TextureView),
            SlotInfo::new("last_colour", SlotType::TextureView),
            SlotInfo::new("normal", SlotType::TextureView),
            SlotInfo::new("position", SlotType::TextureView),
        ]
    }

    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
    }

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        _render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let view_entity = graph.get_input_entity("view")?;
        let gpu_images = world.get_resource::<RenderAssets<Image>>().unwrap();

        let render_attachments = match self.query.get_manual(world, view_entity) {
            Ok(result) => result,
            Err(_) => panic!("Voxel camera missing component!"),
        };

        let colour = gpu_images.get(&render_attachments.colour).unwrap();
        let last_colour = gpu_images.get(&render_attachments.last_colour).unwrap();
        let normal = gpu_images.get(&render_attachments.normal).unwrap();
        let position = gpu_images.get(&render_attachments.position).unwrap();

        let colour = colour.texture_view.clone();
        let last_colour = last_colour.texture_view.clone();
        let normal = normal.texture_view.clone();
        let position = position.texture_view.clone();

        graph
            .set_output("colour", SlotValue::TextureView(colour))
            .unwrap();
        graph
            .set_output("last_colour", SlotValue::TextureView(last_colour))
            .unwrap();
        graph
            .set_output("normal", SlotValue::TextureView(normal))
            .unwrap();
        graph
            .set_output("position", SlotValue::TextureView(position))
            .unwrap();

        Ok(())
    }
}
