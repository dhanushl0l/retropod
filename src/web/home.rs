pub fn render_home<'a>() -> (&'a str, String) {
    (
        "HTTP/1.1 200 OK",
        include_str!("../../assets/html/index.html").to_string(),
    )
}
