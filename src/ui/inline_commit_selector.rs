use ratatui::{
    Frame,
    layout::Rect,
    text::Line,
    widgets::{Block, Borders, Paragraph},
};

use crate::app::{App, FocusedPanel};
use crate::ui::commit_row::{CommitRowSpec, render_commit_row};
use crate::ui::styles;

pub(super) fn render_inline_commit_selector(frame: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.focused_panel == FocusedPanel::CommitSelector;
    let theme = &app.theme;

    let block = Block::default()
        .title(" Commits ")
        .borders(Borders::ALL)
        .style(styles::panel_style(theme))
        .border_style(styles::border_style(theme, focused));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    app.commit_list_viewport_height = inner.height as usize;
    app.commit_list_inner_area = Some(inner);

    let reverse = app.effective_commit_reversed();
    let n = app.review_commits.len();
    let items: Vec<Line> = (0..n)
        .map(|row| {
            let i = crate::commit_order::display_position(row, n, reverse);
            let commit = &app.review_commits[i];
            render_commit_row(&CommitRowSpec {
                commit,
                is_cursor: i == app.commit_list_cursor,
                is_selected: app.is_commit_selected(i),
                is_reviewed: app.is_commit_reviewed_by_viewer(i),
                theme,
            })
        })
        .collect();

    let visible_items: Vec<Line> = items
        .into_iter()
        .skip(app.commit_list_scroll_offset)
        .take(inner.height as usize)
        .collect();

    frame.render_widget(
        Paragraph::new(visible_items).style(styles::panel_style(theme)),
        inner,
    );
}
