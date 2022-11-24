use self::{
    compute::{node::ComputeNode, ComputePlugin},
    trace::{node::TraceNode, TracePlugin},
    voxel_world::VoxelWorldPlugin,
    voxelization::VoxelizationPlugin,
};
use bevy::{
    core_pipeline::{tonemapping::TonemappingNode, upscaling::UpscalingNode},
    prelude::*,
    render::{
        render_graph::{RenderGraph, SlotInfo, SlotType},
        RenderApp,
    },
    ui::UiPassNode,
};

pub mod compute;
pub mod trace;
pub mod voxel_world;
pub mod voxelization;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(VoxelWorldPlugin)
            .add_plugin(TracePlugin)
            .add_plugin(ComputePlugin)
            .add_plugin(VoxelizationPlugin);

        let render_app = match app.get_sub_app_mut(RenderApp) {
            Ok(render_app) => render_app,
            Err(_) => return,
        };

        // build voxel render graph
        let mut voxel_graph = RenderGraph::default();

        let input_node_id =
            voxel_graph.set_input(vec![SlotInfo::new("view_entity", SlotType::Entity)]);
        let main = TraceNode::new(&mut render_app.world);
        let tonemapping = TonemappingNode::new(&mut render_app.world);
        let ui = UiPassNode::new(&mut render_app.world);
        let upscaling = UpscalingNode::new(&mut render_app.world);

        voxel_graph.add_node("main", main);
        voxel_graph.add_node("tonemapping", tonemapping);
        voxel_graph.add_node("ui", ui);
        voxel_graph.add_node("upscaling", upscaling);
        voxel_graph
            .add_slot_edge(input_node_id, "view_entity", "main", "view")
            .unwrap();
        voxel_graph
            .add_slot_edge(input_node_id, "view_entity", "tonemapping", "view")
            .unwrap();
        voxel_graph
            .add_slot_edge(input_node_id, "view_entity", "ui", "view")
            .unwrap();
        voxel_graph
            .add_slot_edge(input_node_id, "view_entity", "upscaling", "view")
            .unwrap();
        voxel_graph.add_node_edge("main", "tonemapping").unwrap();
        voxel_graph.add_node_edge("tonemapping", "ui").unwrap();
        voxel_graph.add_node_edge("ui", "upscaling").unwrap();

        // build voxelization render graph
        // let mut voxelization_graph = RenderGraph::default();

        // let voxelization = VoxelizationNode::new(&mut render_app.world);
        // let input_node_id =
        //     voxelization_graph.set_input(vec![SlotInfo::new("view_entity", SlotType::Entity)]);

        // voxelization_graph.add_node("voxelization", voxelization);
        // voxelization_graph
        //     .add_slot_edge(input_node_id, "view_entity", "voxelization", "view")
        //     .unwrap();

        // insert custom sub graph into the main render graph
        let mut graph = render_app.world.resource_mut::<RenderGraph>();
        graph.add_sub_graph("voxel", voxel_graph);
        // graph.add_sub_graph("voxelization", voxelization_graph);

        // add compute node before camera driver
        graph.add_node("compute", ComputeNode::default());
        graph
            .add_node_edge("compute", bevy::render::main_graph::node::CAMERA_DRIVER)
            .unwrap();
    }
}
