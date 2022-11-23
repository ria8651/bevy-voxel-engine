use self::{trace::{node::TraceNode, TracePlugin}, compute::{ComputePlugin, node::ComputeNode}};
use bevy::{
    core_pipeline::{tonemapping::TonemappingNode, upscaling::UpscalingNode},
    prelude::*,
    render::{
        render_graph::{RenderGraph, SlotInfo, SlotType},
        RenderApp,
    },
    ui::UiPassNode,
};

pub mod trace;
pub mod compute;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(TracePlugin).add_plugin(ComputePlugin);

        let render_app = match app.get_sub_app_mut(RenderApp) {
            Ok(render_app) => render_app,
            Err(_) => return,
        };

        // build custom render graph
        let mut custom_graph = RenderGraph::default();

        let input_node_id =
            custom_graph.set_input(vec![SlotInfo::new("view_entity", SlotType::Entity)]);
        let main = TraceNode::new(&mut render_app.world);
        let tonemapping = TonemappingNode::new(&mut render_app.world);
        let ui = UiPassNode::new(&mut render_app.world);
        let upscaling = UpscalingNode::new(&mut render_app.world);

        custom_graph.add_node("main", main);
        custom_graph.add_node("tonemapping", tonemapping);
        custom_graph.add_node("ui", ui);
        custom_graph.add_node("upscaling", upscaling);
        custom_graph.add_slot_edge(input_node_id, "view_entity", "main", "view").unwrap();
        custom_graph.add_slot_edge(input_node_id, "view_entity", "tonemapping", "view").unwrap();
        custom_graph.add_slot_edge(input_node_id, "view_entity", "ui", "view").unwrap();
        custom_graph.add_slot_edge(input_node_id, "view_entity", "upscaling", "view").unwrap();
        custom_graph.add_node_edge("main", "tonemapping").unwrap();
        custom_graph.add_node_edge("tonemapping", "ui").unwrap();
        custom_graph.add_node_edge("ui", "upscaling").unwrap();

        // insert custom sub graph into the main render graph
        let mut graph = render_app.world.resource_mut::<RenderGraph>();

        graph.add_sub_graph("voxel", custom_graph);

        graph.add_node("compute", ComputeNode::default());
        graph
            .add_node_edge("compute", bevy::render::main_graph::node::CAMERA_DRIVER)
            .unwrap();
    }
}
