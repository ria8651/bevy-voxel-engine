use self::{
    compute::{clear::ClearNode, ComputeResourcesPlugin, rebuild::RebuildNode, automata::AutomataNode, physics::PhysicsNode},
    trace::{node::TraceNode, TracePlugin},
    voxel_world::VoxelWorldPlugin,
    voxelization::VoxelizationPlugin,
};
use bevy::{
    core_pipeline::{tonemapping::TonemappingNode, upscaling::UpscalingNode},
    prelude::*,
    render::{
        main_graph::node::CAMERA_DRIVER,
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
            .add_plugin(VoxelizationPlugin)
            .add_plugin(ComputeResourcesPlugin);

        let render_app = match app.get_sub_app_mut(RenderApp) {
            Ok(render_app) => render_app,
            Err(_) => return,
        };

        // build voxel render graph
        let mut voxel_graph = RenderGraph::default();
        let input_node_id =
            voxel_graph.set_input(vec![SlotInfo::new("view_entity", SlotType::Entity)]);

        // voxel render graph
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

        // voxel render graph compute
        let rebuild = RebuildNode;
        let physics = PhysicsNode;
        voxel_graph.add_node("rebuild", rebuild);
        voxel_graph.add_node("physics", physics);
        voxel_graph.add_node_edge("rebuild", "physics").unwrap();
        voxel_graph.add_node_edge("physics", "main").unwrap();
        
        // main graph compute
        let mut graph = render_app.world.resource_mut::<RenderGraph>();
        let clear = ClearNode;
        let automata = AutomataNode;
        graph.add_node("clear", clear);
        graph.add_node("automata", automata);        
        graph.add_node_edge("clear", "automata").unwrap();
        graph.add_node_edge("automata", CAMERA_DRIVER).unwrap();

        // insert the voxel graph into the main render graph
        graph.add_sub_graph("voxel", voxel_graph);
    }
}