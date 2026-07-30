use std::path::PathBuf;
use gpui::{
    App, Context, Entity, FocusHandle, Focusable, IntoElement,
    ParentElement, Render, SharedString, Styled, Window, EventEmitter, Task,
};
use workspace::{item::ItemEvent, Item};
use ui::{h_flex, prelude::*, v_flex, Label, Button, Icon, IconName};

gpui::actions!(agent_ui, [OmniOrchestratorOpen]);

pub struct AgentOrchestratorView {
    focus_handle: FocusHandle,
    available_skills: Vec<String>,
    nodes: Vec<AgentNode>,
}

struct AgentNode {
    id: usize,
    skill_name: String,
    x: f32,
    y: f32,
}

impl AgentOrchestratorView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let mut view = Self {
            focus_handle: cx.focus_handle(),
            available_skills: Vec::new(),
            nodes: Vec::new(),
        };
        view.load_skills(cx);
        view
    }

    fn load_skills(&mut self, cx: &mut Context<Self>) {
        // Load config from ~/.config/_skills_/
        let mut skills = Vec::new();
        if let Some(home) = std::env::var_os("HOME") {
            let path = PathBuf::from(home).join(".config").join("_skills_");
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_dir() {
                            skills.push(entry.file_name().to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
        skills.sort();
        self.available_skills = skills;
        cx.notify();
    }

    fn compile_to_dag(&self, _cx: &mut Context<Self>) {
        let dag = serde_json::json!({
            "nodes": self.nodes.iter().map(|n| {
                serde_json::json!({
                    "id": n.id,
                    "skill": n.skill_name,
                    "position": { "x": n.x, "y": n.y }
                })
            }).collect::<Vec<_>>(),
            "edges": []
        });
        
        let json_str = serde_json::to_string_pretty(&dag).unwrap_or_default();
        log::info!("Compiled DAG: {}", json_str);
    }
}

impl EventEmitter<ItemEvent> for AgentOrchestratorView {}

impl Item for AgentOrchestratorView {
    type Event = ItemEvent;

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::Sparkle))
    }

    fn telemetry_event_text(&self) -> Option<&'static str> {
        Some("agent_orchestrator")
    }

    fn tab_content_text(&self, _: usize, _cx: &App) -> SharedString {
        "Agent Orchestrator".into()
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

impl Focusable for AgentOrchestratorView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for AgentOrchestratorView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        
        v_flex()
            .size_full()
            .bg(theme.colors().editor_background)
            .child(
                h_flex()
                    .w_full()
                    .p_2()
                    .border_b_1()
                    .border_color(theme.colors().border)
                    .bg(theme.colors().tab_bar_background)
                    .justify_between()
                    .child(Label::new("Omni Smart Agent Design").weight(gpui::FontWeight::BOLD))
                    .child(
                        Button::new("compile_dag", "Compile to Swarm DAG")
                            .on_click(cx.listener(|this, _, _, cx| this.compile_to_dag(cx))),
                    ),
            )
            .child(
                h_flex()
                    .flex_1()
                    .size_full()
                    .child(
                        v_flex()
                            .w_64()
                            .h_full()
                            .border_r_1()
                            .border_color(theme.colors().border)
                            .p_2()
                            .child(Label::new("Available Skills (~/.config/_skills_/)").weight(gpui::FontWeight::BOLD))
                            .child(
                                div().flex_1().children(
                                    self.available_skills.iter().map(|skill| {
                                        div()
                                            .p_1()
                                            .hover(|s| s.bg(theme.colors().element_hover))
                                            .child(Label::new(skill.clone()))
                                            .into_any_element()
                                    })
                                )
                            )
                    )
                    .child(
                        h_flex()
                            .size_full()
                            .child(
                                v_flex()
                                    .h_full()
                                    .w_64()
                                    .bg(cx.theme().colors().surface_background)
                                    .p_4()
                                    .child(Label::new("Canvas Graph UI Placeholder").color(Color::Muted))
                            )
                    )
            )
    }
}
