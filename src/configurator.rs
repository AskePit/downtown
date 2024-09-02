use crate::toml_parser::TomlDoc;
use crate::utils::StrUtils;
use crate::Level;

#[derive(Clone)]
pub(crate) struct Configurator {
    prologue: String,
    epilogue: String,

    image: String,
    link: String,
    latex: String,
    code: String,
    code_inline: String,
    blockquote: String,
    horizontal_line: String,
    paragraph: String,
    bold: String,
    italic: String,
    italic_bold: String,
    strikethrough: String,
    header: String,
    header1: Option<String>,
    header2: Option<String>,
    header3: Option<String>,
    header4: Option<String>,
    header5: Option<String>,
    header6: Option<String>,
    error: String,
}

impl Default for Configurator {
    fn default() -> Self {
        Self {
            prologue: "<html>\n<body>\n".to_string(),
            epilogue: "\n</body>\n</html>".to_string(),
            image: r#"<img src="{src}" alt="{caption}">"#.to_string(),
            link: r#"<a href="{src}">{caption}</a>"#.to_string(),
            latex: r#"<p class="latex">{text}</p>"#.to_string(),
            code: r#"<pre><code class="language-{lang}">{text}</code></pre>"#.to_string(),
            code_inline: r#"<code>{text}</code>"#.to_string(),
            blockquote: r#"<blockquote>{text}</blockquote>"#.to_string(),
            horizontal_line: "<hr>".to_string(),
            paragraph: "<p>{text}</p>".to_string(),
            bold: "<b>{text}</b>".to_string(),
            italic: "<i>{text}</i>".to_string(),
            italic_bold: "<b><i>{text}</i></b>".to_string(),
            strikethrough: "<s>{text}</s>".to_string(),
            header: "<h{level}>{text}</h{level}>".to_string(),
            header1: None,
            header2: None,
            header3: None,
            header4: None,
            header5: None,
            header6: None,
            error: r#"<div class="parse-error">{text}</div>"#.to_string(),
        }
    }
}

impl Configurator {
    pub fn new(config_toml: String) -> Self {
        let default_config = Self::default();

        let doc = TomlDoc::new(&config_toml);

        Self {
            prologue: doc
                .get("page", "prologue")
                .unwrap_or(&default_config.prologue)
                .clone(),
            epilogue: doc
                .get("page", "epilogue")
                .unwrap_or(&default_config.epilogue)
                .clone(),
            image: doc
                .get("tags", "image")
                .unwrap_or(&default_config.image)
                .clone(),
            link: doc
                .get("tags", "link")
                .unwrap_or(&default_config.link)
                .clone(),
            latex: doc
                .get("tags", "latex")
                .unwrap_or(&default_config.latex)
                .clone(),
            code: doc
                .get("tags", "code")
                .unwrap_or(&default_config.code)
                .clone(),
            code_inline: doc
                .get("tags", "code-inline")
                .unwrap_or(&default_config.code_inline)
                .clone(),
            blockquote: doc
                .get("tags", "blockquote")
                .unwrap_or(&default_config.blockquote)
                .clone(),
            horizontal_line: doc
                .get("tags", "horizontal-line")
                .unwrap_or(&default_config.horizontal_line)
                .clone(),
            paragraph: doc
                .get("tags", "paragraph")
                .unwrap_or(&default_config.paragraph)
                .clone(),
            bold: doc
                .get("tags", "bold")
                .unwrap_or(&default_config.bold)
                .clone(),
            italic: doc
                .get("tags", "italic")
                .unwrap_or(&default_config.italic)
                .clone(),
            italic_bold: doc
                .get("tags", "italic-bold")
                .unwrap_or(&default_config.italic_bold)
                .clone(),
            strikethrough: doc
                .get("tags", "strikethrough")
                .unwrap_or(&default_config.strikethrough)
                .clone(),
            header: doc
                .get("tags", "header")
                .unwrap_or(&default_config.header)
                .clone(),
            header1: doc
                .get("tags", "header1")
                .cloned()
                .or(default_config.header1),
            header2: doc
                .get("tags", "header2")
                .cloned()
                .or(default_config.header2),
            header3: doc
                .get("tags", "header3")
                .cloned()
                .or(default_config.header3),
            header4: doc
                .get("tags", "header4")
                .cloned()
                .or(default_config.header4),
            header5: doc
                .get("tags", "header5")
                .cloned()
                .or(default_config.header5),
            header6: doc
                .get("tags", "header6")
                .cloned()
                .or(default_config.header6),
            error: doc
                .get("tags", "error")
                .unwrap_or(&default_config.error)
                .clone(),
        }
    }

    pub fn frame_page(&self, title: &str, page: String) -> String {
        self.prologue.better_replace("{title}", title) + &page + &self.epilogue
    }

    pub fn process_paragraph(&self, text: &str) -> String {
        self.paragraph.better_replace("{text}", text)
    }

    pub fn process_header(&self, level: Level, text: &str) -> String {
        for (l, h) in [
            (1, &self.header1),
            (2, &self.header2),
            (3, &self.header3),
            (4, &self.header4),
            (5, &self.header5),
            (6, &self.header6),
        ] {
            if let Some(h) = h {
                if level == l {
                    return h
                        .better_replace("{text}", text)
                        .better_replace("{level}", &level.to_string());
                }
            }
        }

        self.header
            .better_replace("{text}", text)
            .better_replace("{level}", &level.to_string())
    }

    pub fn process_blockquote(&self, text: &str) -> String {
        self.blockquote.better_replace("{text}", text)
    }

    pub fn process_horizontal_line(&self) -> String {
        self.horizontal_line.clone()
    }

    pub fn process_image(&self, src: &str, caption: &str) -> String {
        self.image
            .better_replace("{caption}", caption)
            .better_replace("{src}", src)
    }

    pub fn process_link(&self, src: &str, caption: &str) -> String {
        self.link
            .better_replace("{caption}", caption)
            .better_replace("{src}", src)
    }

    pub fn process_latex(&self, text: &str) -> String {
        self.latex.better_replace("{text}", text)
    }

    pub fn process_code(&self, lang: &str, text: &str) -> String {
        self.code
            .better_replace("{lang}", lang)
            .better_replace("{text}", text)
    }

    pub fn process_code_inline(&self, text: &str) -> String {
        self.code_inline.better_replace("{text}", text)
    }

    pub fn process_bold(&self, text: &str) -> String {
        self.bold.better_replace("{text}", text)
    }

    pub fn process_italic(&self, text: &str) -> String {
        self.italic.better_replace("{text}", text)
    }

    pub fn process_italic_bold(&self, text: &str) -> String {
        self.italic_bold.better_replace("{text}", text)
    }

    pub fn process_strikethrough(&self, text: &str) -> String {
        self.strikethrough.better_replace("{text}", text)
    }

    pub fn process_error(&self, text: &str) -> String {
        self.error.better_replace("{text}", text)
    }
}
