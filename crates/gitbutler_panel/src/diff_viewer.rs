use gpui::*;
use ui::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LineStatus {
    Added,
    Removed,
    Unchanged,
}

#[derive(Clone)]
pub struct HunkLine {
    pub old_line: Option<u32>,
    pub new_line: Option<u32>,
    pub content: SharedString,
    pub status: LineStatus,
}

#[derive(IntoElement)]
pub struct HunkDiff {
    lines: Vec<HunkLine>,
}

impl HunkDiff {
    pub fn new(lines: Vec<HunkLine>) -> Self {
        Self { lines }
    }
}

impl RenderOnce for HunkDiff {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let status_colors = cx.theme().status();
        v_flex()
            .w_full()
            .children(self.lines.into_iter().map(|line| {
                let bg_color = match line.status {
                    LineStatus::Added => Some(status_colors.created_background),
                    LineStatus::Removed => Some(status_colors.deleted_background),
                    LineStatus::Unchanged => None,
                };
                
                let text_color = match line.status {
                    LineStatus::Added => Some(status_colors.created),
                    LineStatus::Removed => Some(status_colors.deleted),
                    LineStatus::Unchanged => None,
                };

                let mut row = h_flex().w_full();
                if let Some(bg) = bg_color {
                    row = row.bg(bg);
                }

                let line_numbers = h_flex()
                    .w(px(60.))
                    .justify_between()
                    .px_2()
                    .text_color(Color::Muted.color(cx));

                let old_num = line.old_line.map(|l| l.to_string()).unwrap_or_default();
                let new_num = line.new_line.map(|l| l.to_string()).unwrap_or_default();

                let line_numbers = line_numbers
                    .child(div().w_1_2().child(old_num))
                    .child(div().w_1_2().child(new_num));

                let mut content = div()
                    .ml_2()
                    .font_family("JetBrains Mono")
                    .child(line.content.clone());
                    
                if let Some(c) = text_color {
                    content = content.text_color(c);
                }

                row.child(line_numbers).child(content)
            }))
    }
}

#[derive(Clone)]
pub struct Hunk {
    pub lines: Vec<HunkLine>,
}

#[derive(Clone)]
pub struct UnifiedDiff {
    pub file_path: SharedString,
    pub hunks: Vec<Hunk>,
}

#[derive(IntoElement)]
pub struct UnifiedDiffView {
    diff: UnifiedDiff,
}

impl UnifiedDiffView {
    pub fn new(diff: UnifiedDiff) -> Self {
        Self { diff }
    }
}

impl RenderOnce for UnifiedDiffView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        v_flex()
            .w_full()
            .border_1()
            .border_color(cx.theme().colors().border)
            .rounded_md()
            .child(
                h_flex()
                    .w_full()
                    .bg(cx.theme().colors().surface_background)
                    .px_2()
                    .py_1()
                    .child(self.diff.file_path.clone())
            )
            .children(self.diff.hunks.into_iter().map(|hunk| {
                v_flex()
                    .w_full()
                    .border_t_1()
                    .border_color(cx.theme().colors().border)
                    .child(HunkDiff::new(hunk.lines))
            }))
    }
}

pub enum DiffViewState {
    Empty,
    Loading,
    Loaded(but_core::diff::CommitDetails),
    Error(SharedString),
}

pub struct MultiDiffView {
    store: Entity<gitbutler_store::GitButlerStore>,
    state: DiffViewState,
}

impl MultiDiffView {
    pub fn new(store: Entity<gitbutler_store::GitButlerStore>) -> Self {
        Self {
            store,
            state: DiffViewState::Empty,
        }
    }

    pub fn load_commit(&mut self, oid: String, cx: &mut Context<Self>) {
        self.state = DiffViewState::Loading;
        cx.notify();

        let task = self.store.update(cx, |store, cx| store.fetch_commit_details(oid, cx));

        cx.spawn(async move |this, cx| {
            let result = task.await;
            this.update(cx, |this, cx| {
                match result {
                    Ok(details) => {
                        this.state = DiffViewState::Loaded(details);
                    }
                    Err(e) => {
                        this.state = DiffViewState::Error(SharedString::from(e.to_string()));
                    }
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }
}

impl Render for MultiDiffView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content = match &self.state {
            DiffViewState::Empty => div().child("Select a commit to view diff").into_any_element(),
            DiffViewState::Loading => div().child("Loading...").into_any_element(),
            DiffViewState::Error(err) => div().child(err.clone()).text_color(cx.theme().status().deleted).into_any_element(),
            DiffViewState::Loaded(details) => {
                v_flex()
                    .w_full()
                    .gap_1()
                    .children(details.diff_with_first_parent.iter().map(|change| {
                        let path = String::from_utf8_lossy(&change.path).into_owned();
                        
                        let (icon_name, status_color, status_text) = match &change.status {
                            but_core::TreeStatus::Addition { .. } => (IconName::Plus, Color::Success, "Added"),
                            but_core::TreeStatus::Deletion { .. } => (IconName::Trash, Color::Error, "Deleted"),
                            but_core::TreeStatus::Modification { .. } => (IconName::FileDoc, Color::Warning, "Modified"),
                            but_core::TreeStatus::Rename { .. } => (IconName::ArrowRight, Color::Info, "Renamed"),
                        };

                        h_flex()
                            .w_full()
                            .px_2()
                            .py_1()
                            .gap_2()
                            .items_center()
                            .border_b_1()
                            .border_color(cx.theme().colors().border)
                            .hover(|s| s.bg(cx.theme().colors().element_hover))
                            .child(
                                Icon::new(icon_name)
                                    .size(IconSize::Small)
                                    .color(status_color),
                            )
                            .child(
                                Label::new(path)
                                    .size(LabelSize::Small)
                                    .truncate(),
                            )
                            .child(
                                div().ml_auto().child(
                                    Label::new(status_text)
                                        .size(LabelSize::XSmall)
                                        .color(status_color)
                                )
                            )
                    }))
                    .into_any_element()
            }
        };

        v_flex()
            .id("multi-diff-view")
            .w_full()
            .h_full()
            .overflow_y_scroll()
            .p_4()
            .child(content)
    }
}
