use std::collections::HashMap;
use gpui::{
    App, Context, Entity, FocusHandle, Focusable, IntoElement,
    ParentElement, Render, Styled, Window, EventEmitter, SharedString, Task,
    Point, Pixels, point, px,
};
use workspace::{item::ItemEvent, Item};
use ui::{h_flex, prelude::*, v_flex, Label, Icon, IconName, Divider};

use super::dag_compiler::{DagCompiler, DagEdge};
use super::skill_parser::SkillPort;

pub type NodeId = u64;

#[derive(Clone, Debug)]
pub struct CanvasNode {
    pub id: NodeId,
    pub skill_name: String,
    pub label: String,
    pub position: Point<Pixels>,
    pub input_ports: Vec<SkillPort>,
    pub output_ports: Vec<SkillPort>,
    pub prompt_override: String,
    pub model_override: String,
}

impl CanvasNode {
    pub fn new(id: NodeId, skill_name: String, position: Point<Pixels>) -> Self {
        Self {
            id,
            label: skill_name.clone(),
            skill_name,
            position,
            input_ports: Vec::new(),
            output_ports: Vec::new(),
            prompt_override: String::new(),
            model_override: String::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CanvasConnection {
    pub from_node: NodeId,
    pub from_port: String,
    pub to_node: NodeId,
    pub to_port: String,
}

pub struct NodeCanvasEditor {
    nodes: HashMap<NodeId, CanvasNode>,
    connections: Vec<CanvasConnection>,
    next_node_id: NodeId,
    _pan_offset: Point<Pixels>,
    focus_handle: FocusHandle,
    selected_node: Option<NodeId>,
    cycle_errors: Vec<(NodeId, NodeId)>,
}

impl NodeCanvasEditor {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            nodes: HashMap::new(),
            connections: Vec::new(),
            next_node_id: 1,
            _pan_offset: point(px(0.0), px(0.0)),
            focus_handle: cx.focus_handle(),
            selected_node: None,
            cycle_errors: Vec::new(),
        }
    }

    pub fn add_node(&mut self, skill_name: String, position: Point<Pixels>, cx: &mut Context<Self>) -> NodeId {
        let id = self.next_node_id;
        self.next_node_id += 1;
        let node = CanvasNode::new(id, skill_name, position);
        self.nodes.insert(id, node);
        cx.notify();
        id
    }

    pub fn connect(
        &mut self,
        from_node: NodeId,
        from_port: String,
        to_node: NodeId,
        to_port: String,
        cx: &mut Context<Self>,
    ) -> bool {
        let proposed = CanvasConnection {
            from_node,
            from_port: from_port.clone(),
            to_node,
            to_port: to_port.clone(),
        };

        let edges: Vec<DagEdge> = self.connections.iter()
            .map(|c| DagEdge { from: c.from_node, to: c.to_node })
            .chain(std::iter::once(DagEdge { from: from_node, to: to_node }))
            .collect();

        if DagCompiler::has_cycle(&edges) {
            self.cycle_errors = vec![(from_node, to_node)];
            cx.notify();
            return false;
        }

        self.cycle_errors.clear();
        self.connections.push(proposed);
        cx.notify();
        true
    }

    pub fn remove_node(&mut self, node_id: NodeId, cx: &mut Context<Self>) {
        self.nodes.remove(&node_id);
        self.connections.retain(|c| c.from_node != node_id && c.to_node != node_id);
        if self.selected_node == Some(node_id) {
            self.selected_node = None;
        }
        cx.notify();
    }

    pub fn nodes(&self) -> &HashMap<NodeId, CanvasNode> {
        &self.nodes
    }

    pub fn connections(&self) -> &[CanvasConnection] {
        &self.connections
    }

    pub fn selected_node(&self) -> Option<&CanvasNode> {
        self.selected_node.and_then(|id| self.nodes.get(&id))
    }
}

impl Render for NodeCanvasEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let node_count = self.nodes.len();
        let connection_count = self.connections.len();
        let has_cycle_error = !self.cycle_errors.is_empty();

        v_flex()
            .size_full()
            .bg(cx.theme().colors().editor_background)
            // Header
            .child(
                h_flex()
                    .w_full()
                    .p_2()
                    .border_b_1()
                    .border_color(cx.theme().colors().border)
                    .justify_between()
                    .child(Label::new("Omni Smart Agent Design").weight(gpui::FontWeight::BOLD))
                    .child(
                        Label::new(SharedString::from(format!(
                            "Nodes: {} | Connections: {}{}",
                            node_count,
                            connection_count,
                            if has_cycle_error { " | ⚠ Cycle detected!" } else { "" }
                        )))
                        .color(if has_cycle_error { Color::Error } else { Color::Muted })
                        .size(LabelSize::Small),
                    ),
            )
            // Body: Sidebar + Canvas
            .child(
                h_flex()
                    .flex_1()
                    .size_full()
                    // Skill Sidebar
                    .child(
                        v_flex()
                            .w_48()
                            .p_2()
                            .border_r_1()
                            .border_color(cx.theme().colors().border)
                            .child(Label::new("Skills").weight(gpui::FontWeight::BOLD).size(LabelSize::Small))
                            .child(Divider::horizontal())
                            .child(
                                Label::new("Drag skills onto canvas")
                                    .color(Color::Muted)
                                    .size(LabelSize::Small),
                            )
                    )
                    // Canvas Area
                    .child(
                        v_flex()
                            .flex_1()
                            .size_full()
                            .items_center()
                            .justify_center()
                            .child(
                                Label::new("Canvas Editor")
                                    .color(Color::Muted)
                                    .size(LabelSize::Large),
                            )
                            .child(
                                Label::new("(GPUI Path-based Bézier rendering pending)")
                                    .color(Color::Muted)
                                    .size(LabelSize::Small),
                            )
                    )
            )
    }
}

impl Focusable for NodeCanvasEditor {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<ItemEvent> for NodeCanvasEditor {}

impl Item for NodeCanvasEditor {
    type Event = ItemEvent;

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::Sparkle))
    }

    fn telemetry_event_text(&self) -> Option<&'static str> {
        Some("omni_smart_agent_design")
    }

    fn tab_content_text(&self, _: usize, _cx: &App) -> SharedString {
        "Smart Agent Design".into()
    }

    fn clone_on_split(
        &self,
        _workspace_id: Option<workspace::WorkspaceId>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Task<Option<Entity<Self>>> {
        Task::ready(Some(cx.new(|cx| Self::new(cx))))
    }
}
