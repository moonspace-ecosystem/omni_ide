use gpui::{
    App, Context, Entity, FocusHandle, Focusable, IntoElement,
    ParentElement, Render, Styled, Window, EventEmitter, SharedString, Task
};
use workspace::{item::ItemEvent, Item};
use ui::{h_flex, prelude::*, v_flex, Label, Icon, IconName, Divider};

use crate::local_preview_server::{LocalPreviewServer, PreviewServerConfig};

pub struct OmniDesignPreview {
    pub code_content: String,
    preview_server: LocalPreviewServer,
    focus_handle: FocusHandle,
}

impl OmniDesignPreview {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let server = LocalPreviewServer::new(PreviewServerConfig::default());
        Self {
            code_content: String::new(),
            preview_server: server,
            focus_handle: cx.focus_handle(),
        }
    }

    pub fn set_code(&mut self, code: String, cx: &mut Context<Self>) {
        self.preview_server.push_update(&code);
        self.code_content = code;
        cx.notify();
    }

    pub fn server_url(&self) -> Option<String> {
        self.preview_server.url()
    }
}

impl Render for OmniDesignPreview {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let server_status = match self.preview_server.url() {
            Some(url) => format!("Server: {}", url),
            None => "Server: Not started".to_string(),
        };

        v_flex()
            .size_full()
            .bg(cx.theme().colors().editor_background)
            .child(
                h_flex()
                    .w_full()
                    .p_2()
                    .border_b_1()
                    .border_color(cx.theme().colors().border)
                    .justify_between()
                    .child(Label::new("Omni Design Live Preview").weight(gpui::FontWeight::BOLD))
                    .child(Label::new(SharedString::from(server_status)).color(Color::Muted).size(LabelSize::Small)),
            )
            .child(
                v_flex()
                    .flex_1()
                    .size_full()
                    .child(
                        h_flex()
                            .size_full()
                            // Left: Code tab
                            .child(
                                v_flex()
                                    .flex_1()
                                    .p_4()
                                    .child(Label::new("Code").weight(gpui::FontWeight::BOLD).size(LabelSize::Small))
                                    .child(Divider::horizontal())
                                    .child(
                                        Label::new(SharedString::from(
                                            if self.code_content.is_empty() {
                                                "No code generated yet.".to_string()
                                            } else {
                                                self.code_content.clone()
                                            },
                                        ))
                                        .color(Color::Muted)
                                        .size(LabelSize::Small)
                                    )
                            )
                            // Right: Preview (WebView placeholder)
                            .child(
                                v_flex()
                                    .flex_1()
                                    .p_4()
                                    .border_l_1()
                                    .border_color(cx.theme().colors().border)
                                    .child(Label::new("Preview").weight(gpui::FontWeight::BOLD).size(LabelSize::Small))
                                    .child(Divider::horizontal())
                                    .child(
                                        v_flex()
                                            .flex_1()
                                            .items_center()
                                            .justify_center()
                                            .child(
                                                Label::new("Native WebView will render here")
                                                    .color(Color::Muted)
                                                    .size(LabelSize::Large),
                                            )
                                            .child(
                                                Label::new("(Requires wry integration)")
                                                    .color(Color::Muted)
                                                    .size(LabelSize::Small),
                                            )
                                    )
                            )
                    )
            )
    }
}

impl Focusable for OmniDesignPreview {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<ItemEvent> for OmniDesignPreview {}

impl Item for OmniDesignPreview {
    type Event = ItemEvent;

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::Code))
    }

    fn telemetry_event_text(&self) -> Option<&'static str> {
        Some("omni_design_preview")
    }

    fn tab_content_text(&self, _: usize, _cx: &App) -> SharedString {
        "Omni Design".into()
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
