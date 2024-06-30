use serde::Serialize;

#[derive(Serialize)]
pub struct Response<'a> {
    pub class: &'a str,
    pub text: &'a str,
}

// Pulseaudio source
pub struct Source {
    pub name: String,
    pub mute: bool,
}
