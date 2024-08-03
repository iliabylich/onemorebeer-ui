use askama::Template;

#[derive(Template)]
#[template(path = "root.html", escape = "none")]
pub(crate) struct RootTemplate {}
