/// Shortens `content` to fit in `max_width`, appending an ellipsis when needed.
///
/// `measure` returns the width of a shaped candidate string.
pub fn elide(
    content: &str,
    max_width: f32,
    mut measure: impl FnMut(&str) -> f32,
) -> (String, bool) {
    if max_width <= 0.0 {
        return (String::new(), !content.is_empty());
    }

    if measure(content) <= max_width {
        return (content.to_owned(), false);
    }

    const ELLIPSIS: char = '…';
    let ellipsis_width = measure("…");
    if ellipsis_width > max_width {
        return (String::new(), true);
    }

    let mut boundaries: Vec<usize> = content.char_indices().map(|(index, _)| index).collect();
    boundaries.push(content.len());

    let mut fits = 0;
    let mut overflows = boundaries.len() - 1;

    while overflows - fits > 1 {
        let middle = (fits + overflows) / 2;
        let candidate = content[..boundaries[middle]].trim_end();

        if measure(candidate) + ellipsis_width <= max_width {
            fits = middle;
        } else {
            overflows = middle;
        }
    }

    let mut truncated = content[..boundaries[fits]].trim_end().to_owned();
    truncated.push(ELLIPSIS);

    (truncated, true)
}
