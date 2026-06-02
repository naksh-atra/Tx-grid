/// Parse tmux list-panes format output.
pub fn parse_tmux_list_panes_line(line: &str) -> Option<Vec<String>> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.split('\t').map(String::from).collect())
}
