use editor::{Editor, EditorElement};
use gpui::*;
use serde::Deserialize;
use ui::prelude::*;
use workspace::{ModalView, Workspace};

use schemars::JsonSchema;

#[derive(Clone, PartialEq, Deserialize, JsonSchema, Action)]
#[action(namespace = gitbutler)]
pub struct Commit {
    pub branch_name: String,
}

#[derive(Clone, PartialEq, Deserialize, JsonSchema, Action)]
#[action(namespace = gitbutler)]
pub struct Push {
    pub branch_name: String,
}

#[derive(Clone, PartialEq, Deserialize, JsonSchema, Action)]
#[action(namespace = gitbutler)]
pub struct Fetch;

#[derive(Clone, PartialEq, Deserialize, JsonSchema, Action)]
#[action(namespace = gitbutler)]
pub struct CreateBranch;

#[derive(Clone, PartialEq, Deserialize, JsonSchema, Action)]
#[action(namespace = gitbutler)]
pub struct StashChanges;

actions!(
    gitbutler_panel,
    [
        ToggleLeftPanel,
        ToggleRightPanel,
        FocusSearch,
    ]
);

pub struct CommitModal {
    pub text: Entity<Editor>,
    pub branch_name: String,
    pub store: Entity<gitbutler_store::GitButlerStore>,
    focus_handle: FocusHandle,
}

impl EventEmitter<gpui::DismissEvent> for CommitModal {}

impl Focusable for CommitModal {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl ModalView for CommitModal {}

impl CommitModal {
    pub fn register(workspace: &mut Workspace) {
        workspace.register_action(
            |workspace, action: &Commit, window, cx| {
                let panel = workspace.panel::<crate::panel::GitButlerPanel>(cx);
                if let Some(panel) = panel {
                    let store = panel.read(cx).store.clone();
                    let branch_name = action.branch_name.clone();
                    workspace.toggle_modal(window, cx, |window, cx| {
                        CommitModal::new(branch_name, store, window, cx)
                    });
                }
            },
        );
        workspace.register_action(
            |workspace, action: &Push, _window, cx| {
                let panel = workspace.panel::<crate::panel::GitButlerPanel>(cx);
                if let Some(panel) = panel {
                    let store = panel.read(cx).store.clone();
                    let branch_name = action.branch_name.clone();
                    store.update(cx, |store, cx| {
                        store.push(&branch_name, cx);
                    });
                }
            },
        );
        workspace.register_action(
            |workspace, _action: &Fetch, _window, cx| {
                let panel = workspace.panel::<crate::panel::GitButlerPanel>(cx);
                if let Some(panel) = panel {
                    let store = panel.read(cx).store.clone();
                    store.update(cx, |store, cx| {
                        store.fetch(cx);
                    });
                }
            },
        );
    }

    pub fn new(branch_name: String, store: Entity<gitbutler_store::GitButlerStore>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let text = cx.new(|cx| {
            let mut editor = Editor::auto_height(4, 16, window, cx);
            editor.set_placeholder_text("Commit message area...", window, cx);
            editor
        });
        Self {
            text,
            branch_name,
            store,
            focus_handle: cx.focus_handle(),
        }
    }

    fn dismiss(&mut self, _: &menu::Cancel, _: &mut Window, cx: &mut Context<Self>) {
        cx.emit(gpui::DismissEvent);
    }
}

impl Render for CommitModal {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .p_4()
            .w_96()
            .gap_4()
            .bg(cx.theme().colors().elevated_surface_background)
            .rounded_xl()
            .border_1()
            .border_color(cx.theme().colors().border)
            .shadow_lg()
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(Label::new("New Commit").weight(FontWeight::BOLD))
                    .child(
                        Label::new(self.branch_name.clone())
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    ),
            )
            .child(
                div()
                    .w_full()
                    .p_2()
                    .bg(cx.theme().colors().editor_background)
                    .border_1()
                    .border_color(cx.theme().colors().border_variant)
                    .rounded_md()
                    .child(EditorElement::new(&self.text, Default::default())),
            )
            .child(
                h_flex()
                    .justify_end()
                    .gap_2()
                    .child(
                        ui::Button::new("cancel_commit", "Cancel")
                            .style(ui::ButtonStyle::Subtle)
                            .on_click(cx.listener(|_this, _, _window, cx| {
                                cx.emit(gpui::DismissEvent);
                            })),
                    )
                    .child(
                        ui::Button::new("submit_commit", "Commit")
                            .style(ui::ButtonStyle::Filled)
                            .on_click(cx.listener(|this, _, _window, cx| {
                                let text = this.text.read(cx).text(cx);
                                let branch_name = this.branch_name.clone();
                                this.store.update(cx, |store, cx| {
                                    store.commit(&branch_name, &text, cx);
                                });
                                cx.emit(gpui::DismissEvent);
                            })),
                    ),
            )
            .on_action(cx.listener(Self::dismiss))
    }
}
