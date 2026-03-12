use ratatui::Frame;

use crate::app::app::App;

pub fn render(app: &App, frame: &mut Frame) {
    let view = &app.view;
    merge_file_view::render(view, frame);
}

mod merge_file_view {
    use ratatui::{
        Frame,
        layout::{Constraint, Direction, Layout, Rect},
        style::{Color, Modifier, Style},
        text::{Line, Span, Text},
        widgets::{Block, BorderType, Borders, Paragraph},
    };

    use crate::{
        app::merge_file_view::MergeFileView,
        core::{
            model::{Block as MergeBlock, Conflict, ConflictSegment, Resolution},
            renderer::render_conflict,
        },
    };

    const COLOR_OURS: Color = Color::Blue;
    const COLOR_THEIRS: Color = Color::Yellow;
    const COLOR_RESOLVED: Color = Color::Green;
    const COLOR_UNRESOLVED: Color = Color::Red;
    const COLOR_LIGHT_BG: Color = Color::Gray;
    const COLOR_CURRENT_BG: Color = Color::DarkGray;
    const COLOR_LINE_NUM: Color = Color::DarkGray;
    const COLOR_LINE_NUM_CURRENT: Color = Color::Gray;

    pub fn render(merge_file_view: &MergeFileView, frame: &mut Frame) {
        let area = frame.area();
        let outer = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // status bar
                Constraint::Min(0),    // panels
                Constraint::Length(1), // footer
            ])
            .split(area);

        render_statusbar(merge_file_view, frame, outer[0]);
        render_panels(merge_file_view, frame, outer[1]);
        render_footer(frame, outer[2]);
    }

    fn render_statusbar(merge_file_view: &MergeFileView, frame: &mut Frame, area: Rect) {
        let is_dirty = true; // todo move into merge file view
        let dirty = if is_dirty { "[*] " } else { "    " };

        let conflict_info = match merge_file_view.current_conflict_idx() {
            Some(n) => format!(
                "Conflict {}/{}  ({} unresolved)",
                n + 1,
                merge_file_view.num_conflicts(),
                merge_file_view.num_unresolved()
            ),
            None => format!(
                "{} conflicts  ({} unresolved)",
                merge_file_view.num_conflicts(),
                merge_file_view.num_unresolved()
            ),
        };

        let line = Line::from(vec![
            Span::styled(
                format!("  {}", merge_file_view.file_path.display()),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("  {}", dirty), Style::default().fg(Color::Yellow)),
            Span::styled(conflict_info, Style::default().fg(Color::Cyan)),
        ]);

        frame.render_widget(Paragraph::new(line), area);
    }

    enum ConflictSide {
        Ours,
        Theirs,
    }

    fn render_panels(merge_file_view: &MergeFileView, frame: &mut Frame, area: Rect) {
        match merge_file_view.current_conflict() {
            Some(conflict) => {
                let [top, bottom] =
                    Layout::vertical([Constraint::Percentage(35), Constraint::Percentage(65)])
                        .areas(area);

                let [left, right] =
                    Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                        .areas(top);

                render_conflict_side(merge_file_view, ConflictSide::Ours, conflict, frame, left);
                render_conflict_side(
                    merge_file_view,
                    ConflictSide::Theirs,
                    conflict,
                    frame,
                    right,
                );
                //render_theirs(merge_file_view, conflict, frame, right);
                render_merged(merge_file_view, frame, bottom);
            }
            None => render_merged(merge_file_view, frame, area),
        }
    }

    fn render_conflict_side(
        merge_file_view: &MergeFileView,
        conflict_side: ConflictSide,
        conflict: &Conflict,
        frame: &mut Frame,
        area: Rect,
    ) {
        let (title, color, segment) = match conflict_side {
            ConflictSide::Ours => {
                let title = "OURS";
                let color = match &conflict.resolution {
                    Some(resolution) => match resolution {
                        Resolution::Ours => COLOR_RESOLVED,
                        _ => COLOR_CURRENT_BG,
                    },
                    None => COLOR_OURS,
                };
                (title, color, &conflict.ours)
            }
            ConflictSide::Theirs => {
                let title = "THEIRS";
                let color = match &conflict.resolution {
                    Some(resolution) => match resolution {
                        Resolution::Theirs => COLOR_RESOLVED,
                        _ => COLOR_CURRENT_BG,
                    },
                    None => COLOR_THEIRS,
                };
                (title, color, &conflict.theirs)
            }
        };
        render_conflict_segment(merge_file_view, segment, frame, area, title, color);
    }

    fn render_conflict_segment(
        merge_file_view: &MergeFileView,
        conflict_segment: &ConflictSegment,
        frame: &mut Frame,
        area: Rect,
        title: &str,
        color: Color,
    ) {
        let title = match &conflict_segment.tag {
            Some(tag) => format!(" {} ({}) ", title, tag),
            None => format!(" {}", title),
        };

        let block_widget = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(color));

        let inner = block_widget.inner(area);
        frame.render_widget(block_widget, area);

        // scroll: clamp current_block_line to ours length, then center
        let line_count = conflict_segment.lines.len();
        let clamped = merge_file_view
            .current_block_line
            .min(line_count.saturating_sub(1));
        let scroll = clamped.saturating_sub(area.height as usize / 2) as u16;

        let lines: Vec<Line> = conflict_segment
            .lines
            .iter()
            .enumerate()
            .map(|(i, l)| make_styled_line(i + 1, l, color, Color::Reset, i == clamped))
            .collect();

        frame.render_widget(Paragraph::new(Text::from(lines)).scroll((scroll, 0)), inner);
    }

    fn render_merged(merge_file_view: &MergeFileView, frame: &mut Frame, area: Rect) {
        let block_widget = Block::default()
            .title(" MERGED ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::White));

        let inner = block_widget.inner(area);
        frame.render_widget(block_widget, area);

        let (lines, block_lengths) = build_merged_lines(merge_file_view);
        let scroll = centered_scroll(
            &block_lengths,
            merge_file_view.current_block_idx,
            merge_file_view.current_block_line,
            area.height,
        );
        frame.render_widget(Paragraph::new(Text::from(lines)).scroll((scroll, 0)), inner);
    }

    fn build_merged_lines(merge_file_view: &MergeFileView) -> (Vec<Line<'static>>, Vec<usize>) {
        let mut lines = Vec::new();
        let mut block_lengths = Vec::new();
        let mut line_num = 1usize;

        for (block_idx, block) in merge_file_view.merge_file.blocks.iter().enumerate() {
            let start_line_num = line_num;
            let is_current_block = block_idx == merge_file_view.current_block_idx;
            match block {
                MergeBlock::Regular(regular_lines) => {
                    let fg_color = Color::Reset;
                    let bg_color = Color::Reset;
                    for (idx, l) in regular_lines.iter().enumerate() {
                        let cursor = is_current_block && idx == merge_file_view.current_block_line;
                        lines.push(make_styled_line(line_num, l, fg_color, bg_color, cursor));
                        line_num += 1;
                    }
                }
                MergeBlock::Conflict(c) => {
                    let bg_color = if is_current_block {
                        Color::DarkGray
                    } else {
                        Color::Reset
                    };
                    // TODO handle this being Err
                    let conflict_lines = render_conflict(c).unwrap();
                    let fg_color = match c.resolution {
                        Some(_) => COLOR_RESOLVED,
                        None => COLOR_UNRESOLVED,
                    };

                    for (idx, line) in conflict_lines.iter().enumerate() {
                        let cursor = is_current_block && idx == merge_file_view.current_block_line;
                        lines.push(make_styled_line(line_num, line, fg_color, bg_color, cursor));
                        line_num += 1;
                    }
                }
            }
            block_lengths.push(line_num - start_line_num);
        }
        (lines, block_lengths)
    }

    /// Given a layout and cursor, compute the scroll offset that centers the
    /// cursor line vertically within a viewport of `viewport_height` rows.
    ///
    /// If the cursor's block_line exceeds this view's block length (i.e. the
    /// cursor is in a part of the global layout that doesn't exist here), we
    /// clamp to the last line of that block — the view just sits there.
    fn centered_scroll(
        block_lengths: &[usize],
        block_idx: usize,
        block_line: usize,
        viewport_height: u16,
    ) -> u16 {
        let block_start_line = &block_lengths[..block_idx].iter().sum();
        let block_len = block_lengths[block_idx];
        let clamped = if block_len == 0 {
            0
        } else {
            block_line.min(block_len - 1)
        };
        let my_line = block_start_line + clamped;
        let half = (viewport_height / 2) as usize;
        my_line.saturating_sub(half) as u16
    }

    fn make_styled_line(
        line_num: usize,
        content: &str,
        fg_color: Color,
        bg_color: Color,
        cursor: bool,
    ) -> Line<'static> {
        let gutter = if cursor { "▶" } else { " " };
        let line_num_color = if bg_color == COLOR_CURRENT_BG {
            COLOR_LINE_NUM_CURRENT
        } else {
            COLOR_LINE_NUM
        };
        Line::from(vec![
            Span::styled(
                format!("{:>4}  ", line_num),
                Style::default().fg(line_num_color).bg(bg_color),
            ),
            Span::styled(
                format!("{} ", gutter),
                Style::default().fg(fg_color).bg(bg_color),
            ),
            Span::styled(
                content.to_string(),
                Style::default().fg(fg_color).bg(bg_color),
            ),
        ])
    }

    fn render_footer(frame: &mut Frame, area: Rect) {
        let line = Line::from(vec![
            key(" n/p "),
            desc(" next/prev conflict  "),
            key(" C-n/C-p "),
            desc(" next/prev unresolved  "),
            key(" j/k "),
            desc(" scroll  "),
            key("  o  "),
            desc(" ours  "),
            key("  t  "),
            desc(" theirs  "),
            key("  c  "),
            desc(" clear  "),
            key("  q  "),
            desc(" quit  "),
        ]);
        frame.render_widget(Paragraph::new(line), area);
    }

    fn key(s: &str) -> Span<'static> {
        Span::styled(
            s.to_string(),
            Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
    }

    fn desc(s: &str) -> Span<'static> {
        Span::styled(s.to_string(), Style::default().fg(COLOR_LIGHT_BG))
    }
}
