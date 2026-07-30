use std::fmt::Write as _;
use std::path::Path;
use std::sync::Arc;

use editor::{Editor, EditorMode, MultiBuffer};
use gpui::{
    App, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement, ParentElement, Render,
    SharedString, Styled, Task, WeakEntity, Window,
};
use language::Buffer;
use language_model::{
    LanguageModelRegistry, LanguageModelRequest, LanguageModelRequestMessage, MessageContent,
    Role,
};
use ui::{h_flex, prelude::*, v_flex, Button, Divider, Icon, IconName, Label};
use workspace::{item::ItemEvent, Item, Workspace};

use crate::omni_chat_store::{
    ChatSessionMetadata, ChatSessionStore, ContextSource, ContextSourceType,
};
use crate::omni_rag::RAGIndex;

gpui::actions!(agent_ui, [OmniChatOpen]);

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub sender: Role,
    pub content: String,
}

pub struct OmniChat {
    focus_handle: FocusHandle,
    workspace: WeakEntity<Workspace>,
    input_editor: Entity<Editor>,
    messages: Vec<ChatMessage>,
    context_sources: Vec<ContextSource>,
    rag_index: Arc<parking_lot::Mutex<RAGIndex>>,
    store: Arc<ChatSessionStore>,
    session_id: String,
    is_responding: bool,
    context_input_editor: Entity<Editor>,
}

impl OmniChat {
    pub fn new(
        workspace: WeakEntity<Workspace>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let workspace_root = workspace
            .upgrade()
            .and_then(|w| {
                w.read(cx)
                    .project()
                    .read(cx)
                    .visible_worktrees(cx)
                    .next()
            })
            .map(|wt| wt.read(cx).abs_path().to_path_buf())
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        let store = Arc::new(ChatSessionStore::new(workspace_root));
        let session_id = uuid::Uuid::new_v4().to_string();

        let input_editor = cx.new(|cx| {
            let buffer = cx.new(|cx| Buffer::local("", cx));
            let buffer = cx.new(|cx| MultiBuffer::singleton(buffer, cx));
            let mut editor = Editor::new(
                EditorMode::AutoHeight {
                    min_lines: 1,
                    max_lines: Some(8),
                },
                buffer,
                None,
                window,
                cx,
            );
            editor.set_placeholder_text("Type a message to brainstorm...", window, cx);
            editor
        });

        let context_input_editor = cx.new(|cx| {
            let buffer = cx.new(|cx| Buffer::local("", cx));
            let buffer = cx.new(|cx| MultiBuffer::singleton(buffer, cx));
            let mut editor = Editor::new(EditorMode::SingleLine, buffer, None, window, cx);
            editor.set_placeholder_text("Enter file path or URL...", window, cx);
            editor
        });

        Self {
            focus_handle: cx.focus_handle(),
            workspace,
            input_editor,
            messages: Vec::new(),
            context_sources: Vec::new(),
            rag_index: Arc::new(parking_lot::Mutex::new(RAGIndex::new())),
            store,
            session_id,
            is_responding: false,
            context_input_editor,
        }
    }

    fn send_message(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_responding {
            return;
        }

        let user_text = self.input_editor.read(cx).text(cx).trim().to_string();
        if user_text.is_empty() {
            return;
        }

        self.input_editor
            .update(cx, |editor, cx| editor.set_text("", window, cx));

        self.messages.push(ChatMessage {
            sender: Role::User,
            content: user_text.clone(),
        });

        self.is_responding = true;
        cx.notify();

        let rag = self.rag_index.clone();
        let top_chunks = cx
            .background_executor()
            .spawn(async move { rag.lock().query(&user_text, 3) });

        cx.spawn(async move |this, cx| {
            let chunks = top_chunks.await;

            let model = cx
                .update(|cx| LanguageModelRegistry::read_global(cx).default_model());

            let Some(configured_model) = model else {
                this.update(cx, |this, cx| {
                    this.messages.push(ChatMessage {
                        sender: Role::Assistant,
                        content: "Error: No language model active or configured. Please configure one in settings.".to_string(),
                    });
                    this.is_responding = false;
                    cx.notify();
                })
                .ok();
                return;
            };

            let mut messages = Vec::new();

            if !chunks.is_empty() {
                let context_str = chunks
                    .iter()
                    .map(|c| format!("Source: {}\nContent:\n{}", c.source_id, c.text))
                    .collect::<Vec<_>>()
                    .join("\n\n---\n\n");

                messages.push(LanguageModelRequestMessage {
                    role: Role::System,
                    content: vec![MessageContent::Text(format!(
                        "You are an AI research partner inside Omni IDE. You are helping brainstorm, document requirements, and design solutions.\n\
                         Below is relevant context retrieved from indexed sources. Use this context to inform your responses:\n\n{}",
                        context_str
                    ))],
                    cache: false,
                    reasoning_details: None,
                });
            } else {
                messages.push(LanguageModelRequestMessage {
                    role: Role::System,
                    content: vec![MessageContent::Text(
                        "You are an AI research partner inside Omni IDE. You are helping brainstorm, document requirements, and design solutions.".to_string()
                    )],
                    cache: false,
                    reasoning_details: None,
                });
            }

            let history = this
                .update(cx, |this, _| this.messages.clone())
                .unwrap_or_default();
            for msg in history {
                messages.push(LanguageModelRequestMessage {
                    role: msg.sender,
                    content: vec![MessageContent::Text(msg.content)],
                    cache: false,
                    reasoning_details: None,
                });
            }

            let request = LanguageModelRequest {
                messages,
                ..Default::default()
            };

            let mut response_text = String::new();
            let stream_result = configured_model
                .model
                .stream_completion_text(request, cx)
                .await;

            if let Ok(stream) = stream_result {
                let mut stream = stream.stream;
                use futures::StreamExt as _;

                this.update(cx, |this, cx| {
                    this.messages.push(ChatMessage {
                        sender: Role::Assistant,
                        content: String::new(),
                    });
                    cx.notify();
                })
                .ok();

                while let Some(chunk) = stream.next().await {
                    if let Ok(text) = chunk {
                        response_text.push_str(&text);
                        let response_text = response_text.clone();
                        this.update(cx, |this, cx| {
                            if let Some(last) = this.messages.last_mut() {
                                last.content = response_text;
                            }
                            cx.notify();
                        })
                        .ok();
                    }
                }
            } else {
                this.update(cx, |this, cx| {
                    this.messages.push(ChatMessage {
                        sender: Role::Assistant,
                        content: "Error: Failed to stream completion from the language model.".to_string(),
                    });
                    cx.notify();
                })
                .ok();
            }

            this.update(cx, |this, cx| {
                this.is_responding = false;

                let meta = ChatSessionMetadata {
                    id: this.session_id.clone(),
                    title: this
                        .messages
                        .first()
                        .map(|m| m.content.chars().take(30).collect())
                        .unwrap_or_else(|| "New Session".to_string()),
                    created_at: 0,
                    updated_at: 0,
                    context_sources: this.context_sources.clone(),
                };
                let _ = this.store.save_session(&meta);
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn add_context_source(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let source_path_or_url = self.context_input_editor.read(cx).text(cx).trim().to_string();
        if source_path_or_url.is_empty() {
            return;
        }

        self.context_input_editor
            .update(cx, |editor, cx| editor.set_text("", window, cx));

        let source_type = if source_path_or_url.starts_with("http://")
            || source_path_or_url.starts_with("https://")
        {
            ContextSourceType::Web
        } else {
            ContextSourceType::File
        };

        let id = uuid::Uuid::new_v4().to_string();
        let title = Path::new(&source_path_or_url)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| source_path_or_url.clone());

        let source = ContextSource {
            id: id.clone(),
            source_type: source_type.clone(),
            uri: source_path_or_url.clone(),
            title: title.clone(),
            indexed_at: 0,
            chunk_count: 0,
        };

        self.context_sources.push(source);
        cx.notify();

        let rag = self.rag_index.clone();

        cx.spawn(async move |this, cx| {
            let mut content = String::new();
            match source_type {
                ContextSourceType::File => {
                    if let Ok(text) = std::fs::read_to_string(&source_path_or_url) {
                        content = text;
                    }
                }
                ContextSourceType::Web => {
                    // Web fetch omitted to keep compilation simple.
                }
                _ => {}
            }

            if !content.is_empty() {
                let mut index = rag.lock();
                index.index_document(&source_path_or_url, &content);

                this.update(cx, |this, cx| {
                    if let Some(src) = this.context_sources.iter_mut().find(|s| s.id == id) {
                        src.chunk_count = content.split_whitespace().count() / 150;
                    }
                    cx.notify();
                })
                .ok();
            }
        })
        .detach();
    }

    fn transfer_context(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.messages.is_empty() {
            return;
        }

        let history = self.messages.clone();
        let model = LanguageModelRegistry::read_global(cx).default_model();

        cx.spawn(async move |this, cx| {
            let Some(configured_model) = model else {
                return;
            };

            let mut summary_request = LanguageModelRequest::default();
            summary_request.messages.push(LanguageModelRequestMessage {
                role: Role::System,
                content: vec![MessageContent::Text(
                    "You are a technical coordinator. Summarize the following brainstorming and requirements session into a concise, actionable Executive Summary for a software designer to build a UI prototype. Focus on specific screens, layouts, states, and functionality discussed.".to_string()
                )],
                cache: false,
                reasoning_details: None,
            });

            let mut session_str = String::new();
            for msg in history {
                let _ = write!(session_str, "{}: {}\n\n", msg.sender, msg.content);
            }

            summary_request.messages.push(LanguageModelRequestMessage {
                role: Role::User,
                content: vec![MessageContent::Text(session_str)],
                cache: false,
                reasoning_details: None,
            });

            let mut summary = String::new();
            let stream_result = configured_model
                .model
                .stream_completion_text(summary_request, &cx)
                .await;

            if let Ok(stream) = stream_result {
                let mut stream = stream.stream;
                use futures::StreamExt as _;
                while let Some(chunk) = stream.next().await {
                    if let Ok(text) = chunk {
                        summary.push_str(&text);
                    }
                }
            }

            if summary.is_empty() {
                summary = "Failed to generate executive summary.".to_string();
            }

            this.update(cx, |this, cx| {
                let _ = this.workspace.update_in(cx, |workspace, window, cx| {
                    let preview = cx.new(|cx| {
                        let mut p = crate::omni_design_preview::OmniDesignPreview::new(cx);
                        p.set_code(summary, cx);
                        p
                    });
                    workspace.add_item_to_active_pane(
                        Box::new(preview),
                        None,
                        true,
                        window,
                        cx,
                    );
                });
            })
            .ok();
        })
        .detach();
    }
}

impl Render for OmniChat {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        h_flex()
            .size_full()
            .bg(theme.colors().editor_background)
            // Left Column: Chat Area
            .child(
                v_flex()
                    .flex_1()
                    .size_full()
                    .p_4()
                    .child(
                        h_flex()
                            .w_full()
                            .justify_between()
                            .items_center()
                            .child(
                                Label::new("Omni Chat (Deep Research)")
                                    .weight(gpui::FontWeight::BOLD),
                            )
                            .child(
                                Button::new("transfer_context", "Transfer Context").on_click(
                                    cx.listener(|this, _, window, cx| {
                                        this.transfer_context(window, cx)
                                    }),
                                ),
                            ),
                    )
                    .child(Divider::horizontal())
                    // Messages History list
                    .child(
                        v_flex()
                            .flex_1()
                            .children(self.messages.iter().map(|msg| {
                                let (sender_name, sender_color) = match msg.sender {
                                    Role::User => ("You", Color::Info),
                                    Role::Assistant => ("Omni Brain", Color::Success),
                                    Role::System => ("System", Color::Muted),
                                };
                                v_flex()
                                    .w_full()
                                    .bg(theme.colors().surface_background)
                                    .p_3()
                                    .rounded_md()
                                    .border_1()
                                    .border_color(theme.colors().border)
                                    .child(
                                        Label::new(sender_name)
                                            .weight(gpui::FontWeight::BOLD)
                                            .color(sender_color),
                                    )
                                    .child(Divider::horizontal())
                                    .child(Label::new(msg.content.clone()))
                            })),
                    )
                    .child(Divider::horizontal())
                    // Input Area
                    .child(
                        v_flex()
                            .gap_2()
                            .child(self.input_editor.clone())
                            .child(
                                h_flex().justify_end().child(
                                    Button::new(
                                        "send_message",
                                        if self.is_responding {
                                            "Thinking..."
                                        } else {
                                            "Send"
                                        },
                                    )
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.send_message(window, cx)
                                    })),
                                ),
                            ),
                    ),
            )
            // Right Column: Context Sources
            .child(
                v_flex()
                    .w_64()
                    .size_full()
                    .p_4()
                    .border_l_1()
                    .border_color(theme.colors().border)
                    .child(Label::new("Context Sources").weight(gpui::FontWeight::BOLD))
                    .child(Divider::horizontal())
                    // Context index input
                    .child(
                        v_flex()
                            .gap_2()
                            .child(self.context_input_editor.clone())
                            .child(
                                Button::new("add_context", "Index Source").on_click(
                                    cx.listener(|this, _, window, cx| {
                                        this.add_context_source(window, cx)
                                    }),
                                ),
                            ),
                    )
                    .child(Divider::horizontal())
                    // Context Sources List
                    .child(
                        v_flex()
                            .flex_1()
                            .children(if self.context_sources.is_empty() {
                                vec![Label::new("No context indexed.")
                                    .color(Color::Muted)
                                    .into_any_element()]
                            } else {
                                self.context_sources
                                    .iter()
                                    .map(|src| {
                                        let icon = match src.source_type {
                                            ContextSourceType::Web => IconName::Link,
                                            ContextSourceType::File => IconName::File,
                                            _ => IconName::FileDoc,
                                        };
                                        h_flex()
                                            .gap_2()
                                            .items_center()
                                            .p_1()
                                            .hover(|s| s.bg(theme.colors().element_hover))
                                            .child(Icon::new(icon).size(IconSize::Small))
                                            .child(
                                                v_flex()
                                                    .child(
                                                        Label::new(src.title.clone()).truncate(),
                                                    )
                                                    .child(
                                                        Label::new(format!("{} chunks", src.chunk_count))
                                                            .size(LabelSize::XSmall)
                                                            .color(Color::Muted),
                                                    ),
                                            )
                                            .into_any_element()
                                    })
                                    .collect::<Vec<_>>()
                            }),
                    ),
            )
    }
}

impl Focusable for OmniChat {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<ItemEvent> for OmniChat {}

impl Item for OmniChat {
    type Event = ItemEvent;

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::Chat))
    }

    fn telemetry_event_text(&self) -> Option<&'static str> {
        Some("omni_chat")
    }

    fn tab_content_text(&self, _: usize, _cx: &App) -> SharedString {
        "Omni Chat".into()
    }

    fn clone_on_split(
        &self,
        _workspace_id: Option<workspace::WorkspaceId>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Task<Option<Entity<Self>>> {
        let ws = self.workspace.clone();
        Task::ready(Some(cx.new(|cx| Self::new(ws, window, cx))))
    }
}
