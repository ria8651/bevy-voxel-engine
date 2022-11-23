use bevy::{
    core_pipeline::{tonemapping::TonemappingNode, upscaling::UpscalingNode},
    prelude::*,
    render::{
        render_graph::{RenderGraph, SlotInfo, SlotType},
        RenderApp,
    },
};
use self::trace::{TracePlugin, node::TraceNode};

mod trace;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(TracePlugin);

        let render_app = match app.get_sub_app_mut(RenderApp) {
            Ok(render_app) => render_app,
            Err(_) => return,
        };

        // build custom render graph
        let mut custom_graph = RenderGraph::default();

        let main = TraceNode::new(&mut render_app.world);
        let tonemapping = TonemappingNode::new(&mut render_app.world);
        let upscaling = UpscalingNode::new(&mut render_app.world);
        let input_node_id =
            custom_graph.set_input(vec![SlotInfo::new("view_entity", SlotType::Entity)]);

        custom_graph.add_node("main", main);
        custom_graph.add_node("tonemapping", tonemapping);
        custom_graph.add_node("upscaling", upscaling);
        custom_graph
            .add_slot_edge(input_node_id, "view_entity", "main", "view")
            .unwrap();
        custom_graph
            .add_slot_edge(input_node_id, "view_entity", "tonemapping", "view")
            .unwrap();
        custom_graph
            .add_slot_edge(input_node_id, "view_entity", "upscaling", "view")
            .unwrap();
        custom_graph.add_node_edge("main", "tonemapping").unwrap();
        custom_graph
            .add_node_edge("tonemapping", "upscaling")
            .unwrap();

        // insert custom sub graph into the main render graph
        let mut graph = render_app.world.get_resource_mut::<RenderGraph>().unwrap();
        graph.add_sub_graph("voxel", custom_graph);
    }
}
