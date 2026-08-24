use pulldown_cmark::{html, Options, Parser};

pub fn render_markdown(body: &str) -> String {
    let parser = Parser::new_ext(body, Options::all());
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    format!("<article>\n{html_output}</article>")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_heading() {
        let html = render_markdown("# Hello");
        assert!(html.contains("<h1>Hello</h1>"));
        assert!(html.starts_with("<article>"));
    }

    #[test]
    fn renders_paragraph() {
        let html = render_markdown("Some text.");
        assert!(html.contains("<p>Some text.</p>"));
    }
}
