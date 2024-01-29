pub fn escape(s: &str) -> String {
    let s = s.to_owned();
    let s = s.replace("\\", "\\\\");
    let s = s.replace("\"", "\\\"");
    s
}