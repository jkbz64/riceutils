use serde::Serialize;

#[derive(Serialize)]
pub struct Response<'a> {
    pub class: &'a str,
    pub text: &'a str,
}
