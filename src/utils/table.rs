fn widths(headers: &[&str], rows: &[Vec<String>]) -> Vec<usize> {
    let mut w = headers.iter().map(|h| h.len()).collect::<Vec<_>>();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < w.len() {
                w[i] = w[i].max(cell.chars().count());
            }
        }
    }
    w
}

pub fn render_table(headers: &[&str], rows: &[Vec<String>]) -> String {
    let widths = widths(headers, rows);
    let sep = format!(
        "┼{}┼",
        widths
            .iter()
            .map(|w| "─".repeat(*w + 2))
            .collect::<Vec<_>>()
            .join("┼")
    );

    let mut out = String::new();
    out.push_str(&format!(
        "┌{}┐\n",
        widths
            .iter()
            .map(|w| "─".repeat(*w + 2))
            .collect::<Vec<_>>()
            .join("┬")
    ));
    out.push_str(&format_row(
        &headers.iter().map(|h| h.to_string()).collect::<Vec<_>>(),
        &widths,
    ));
    out.push('\n');
    out.push_str(&sep);
    out.push('\n');

    for (idx, row) in rows.iter().enumerate() {
        out.push_str(&format_row(row, &widths));
        if idx + 1 < rows.len() {
            out.push('\n');
        }
    }

    if !rows.is_empty() {
        out.push('\n');
    }
    out.push_str(&format!(
        "└{}┘",
        widths
            .iter()
            .map(|w| "─".repeat(*w + 2))
            .collect::<Vec<_>>()
            .join("┴")
    ));
    out
}

fn format_row(row: &[String], widths: &[usize]) -> String {
    let mut out = String::new();
    out.push('│');
    for (i, width) in widths.iter().enumerate() {
        let cell = row.get(i).cloned().unwrap_or_default();
        let pad = width.saturating_sub(cell.chars().count());
        out.push(' ');
        out.push_str(&cell);
        out.push_str(&" ".repeat(pad + 1));
        out.push('│');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::render_table;

    #[test]
    fn table_renders_borders() {
        let out = render_table(&["A", "B"], &[vec!["1".into(), "22".into()]]);
        assert!(out.contains("┌"));
        assert!(out.contains("└"));
        assert!(out.contains("22"));
    }
}
