use std::path::Path;

pub fn path_str(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

pub fn map_still(label: &str, out: &Path) -> Vec<String> {
    vec![
        "-map".into(),
        label.into(),
        "-frames:v".into(),
        "1".into(),
        path_str(out),
    ]
}
