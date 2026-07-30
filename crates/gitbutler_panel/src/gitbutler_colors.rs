use gpui::Hsla;
use theme::Theme;

/// Provides GitButler-specific status colors as an extension to the Zed theme.
pub trait GitButlerColors {
    fn gitbutler_virtual_branch(&self) -> Hsla;
    fn gitbutler_applied(&self) -> Hsla;
    fn gitbutler_unapplied(&self) -> Hsla;
    fn gitbutler_active(&self) -> Hsla;
    fn gitbutler_conflict(&self) -> Hsla;
    fn gitbutler_local_commit(&self) -> Hsla;
    fn gitbutler_pushed_commit(&self) -> Hsla;
    fn gitbutler_integrated_commit(&self) -> Hsla;
    fn gitbutler_drag_indicator(&self) -> Hsla;
    fn gitbutler_drop_target(&self) -> Hsla;
}

impl GitButlerColors for Theme {
    fn gitbutler_virtual_branch(&self) -> Hsla {
        self.status().info
    }

    fn gitbutler_applied(&self) -> Hsla {
        self.status().success
    }

    fn gitbutler_unapplied(&self) -> Hsla {
        self.status().modified
    }

    fn gitbutler_active(&self) -> Hsla {
        self.status().info
    }

    fn gitbutler_conflict(&self) -> Hsla {
        self.status().conflict
    }

    fn gitbutler_local_commit(&self) -> Hsla {
        self.status().info
    }

    fn gitbutler_pushed_commit(&self) -> Hsla {
        self.status().success
    }

    fn gitbutler_integrated_commit(&self) -> Hsla {
        self.status().hint
    }

    fn gitbutler_drag_indicator(&self) -> Hsla {
        self.status().info
    }

    fn gitbutler_drop_target(&self) -> Hsla {
        gpui::hsla(0.58, 0.7, 0.5, 0.15)
    }
}
