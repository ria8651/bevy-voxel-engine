use self::{
    attachments::{AttachmentsNode, AttachmentsPlugin},
    compute::{
        animation::AnimationNode, automata::AutomataNode, clear::ClearNode, physics::PhysicsNode,
        rebuild::RebuildNode, ComputeResourcesPlugin,
    },
    trace::{TraceNode, TracePlugin},
    voxel_world::VoxelWorldPlugin,
    voxelization::VoxelizationPlugin,
};
use bevy::{
    core_pipeline::{fxaa::FxaaNode, tonemapping::TonemappingNode, upscaling::UpscalingNode},
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        main_graph::node::CAMERA_DRIVER,
        render_graph::{RenderGraph, ViewNodeRunner},
        RenderApp,
    },
    ui::UiPassNode,
};

pub mod attachments;
pub mod compute;
pub mod trace;
pub mod voxel_world;
pub mod voxelization;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RenderGraphSettings::default())
            .add_plugins(ExtractResourcePlugin::<RenderGraphSettings>::default())
            .add_plugins(AttachmentsPlugin)
            .add_plugins(VoxelWorldPlugin)
            .add_plugins(TracePlugin)
            .add_plugins(VoxelizationPlugin)
            .add_plugins(ComputeResourcesPlugin);
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        let render_world = &mut render_app.world;

        // Build voxel render graph
        let mut voxel_graph = RenderGraph::default();

        // Voxel render graph
        let attachments = AttachmentsNode::new(render_world);
        let trace = TraceNode::new(render_world);
        //let bloom = BloomNode::new(&mut render_app.world);
        let tonemapping = TonemappingNode::from_world(render_world);
        let fxaa = FxaaNode::from_world(render_world);
        let ui = UiPassNode::new(render_world);
        let upscaling = UpscalingNode::from_world(render_world);

        voxel_graph.add_node("attachments", attachments);
        voxel_graph.add_node("trace", trace);
        voxel_graph.add_node(
            "tonemapping",
            ViewNodeRunner::new(tonemapping, render_world),
        );
        voxel_graph.add_node("fxaa", ViewNodeRunner::new(fxaa, render_world));
        voxel_graph.add_node("ui", ui);
        voxel_graph.add_node("upscaling", ViewNodeRunner::new(upscaling, render_world));

        voxel_graph.add_node_edge("trace", "tonemapping");
        voxel_graph.add_node_edge("tonemapping", "fxaa");
        voxel_graph.add_node_edge("fxaa", "ui");
        voxel_graph.add_node_edge("ui", "upscaling");

        voxel_graph.add_slot_edge("attachments", "normal", "trace", "normal");
        voxel_graph.add_slot_edge("attachments", "position", "trace", "position");

        // Voxel render graph compute
        voxel_graph.add_node("rebuild", RebuildNode);
        voxel_graph.add_node("physics", PhysicsNode);

        voxel_graph.add_node_edge("rebuild", "physics");
        voxel_graph.add_node_edge("physics", "trace");

        // Main graph compute
        let mut graph = render_world.resource_mut::<RenderGraph>();

        graph.add_node("clear", ClearNode);
        graph.add_node("automata", AutomataNode);
        graph.add_node("animation", AnimationNode);

        graph.add_node_edge("clear", "automata");
        graph.add_node_edge("automata", "animation");
        graph.add_node_edge("animation", CAMERA_DRIVER);

        // Insert the voxel graph into the main render graph
        graph.add_sub_graph("voxel", voxel_graph);
    }
}

#[derive(Resource, Clone, ExtractResource)]
pub struct RenderGraphSettings {
    pub clear: bool,
    pub automata: bool,
    pub animation: bool,
    pub voxelization: bool,
    pub rebuild: bool,
    pub physics: bool,
    pub trace: bool,
}

impl Default for RenderGraphSettings {
    fn default() -> Self {
        Self {
            clear: true,
            automata: true,
            animation: true,
            voxelization: true,
            rebuild: true,
            physics: true,
            trace: true,
        }
    }
}
