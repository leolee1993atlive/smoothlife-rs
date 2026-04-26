mod smoothlife_bevy;
mod smoothlife_core;
mod smoothlife_term;

fn main() {
    let mode = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "term".to_string());

    match mode.as_str() {
        "bevy" => smoothlife_bevy::run(),
        _ => smoothlife_term::run(),
    }
}
