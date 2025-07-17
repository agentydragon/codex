#![allow(
    clippy::extend_with_drain,
    unused_mut,
    deprecated,
    clippy::manual_div_ceil,
    clippy::uninlined_format_args
)]

use std::sync::LazyLock;

use pulldown_cmark::CodeBlockKind;
use pulldown_cmark::CowStr;
use pulldown_cmark::Event;
use pulldown_cmark::HeadingLevel;
use pulldown_cmark::Options;
use pulldown_cmark::Parser;
use pulldown_cmark::Tag;
use pulldown_cmark::TagEnd;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::text::Text;

#[cfg(feature = "highlight-code")]
use ansi_to_tui::IntoText;
#[cfg(feature = "highlight-code")]
use syntect::easy::HighlightLines;
#[cfg(feature = "highlight-code")]
use syntect::highlighting::ThemeSet;
#[cfg(feature = "highlight-code")]
use syntect::parsing::SyntaxSet;
#[cfg(feature = "highlight-code")]
use syntect::util::LinesWithEndings;
#[cfg(feature = "highlight-code")]
use syntect::util::as_24_bit_terminal_escaped;

use crate::style::parse_style;
use codex_core::config_types::Styles;

// Lazy statics for syntax highlighting (only loaded when needed)
#[cfg(feature = "highlight-code")]
static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);
#[cfg(feature = "highlight-code")]
static THEME_SET: LazyLock<ThemeSet> = LazyLock::new(ThemeSet::load_defaults);

/// Custom markdown renderer that properly handles code blocks and provides configurable styling
pub struct MarkdownRenderer<'a> {
    /// Output text being built
    text: Text<'static>,

    /// Current line being built
    current_line: Line<'static>,

    /// Stack of inline styles (for nested formatting like bold+italic)
    style_stack: Vec<Style>,

    /// Line prefixes for things like blockquotes ("> ")
    line_prefixes: Vec<Span<'static>>,

    /// Whether we need to add a newline before the next content
    needs_newline: bool,

    /// Current list nesting with indices (None for unordered, Some(n) for ordered)
    list_stack: Vec<Option<u64>>,

    /// Code block highlighting state
    #[cfg(feature = "highlight-code")]
    code_highlighter: Option<HighlightLines<'a>>,

    /// Configuration styles from the user's config
    styles: &'a Styles,
}

impl<'a> MarkdownRenderer<'a> {
    /// Create a new markdown renderer with the given style configuration
    pub fn new(styles: &'a Styles) -> Self {
        Self {
            text: Text::default(),
            current_line: Line::default(),
            style_stack: vec![Style::default()],
            line_prefixes: vec![],
            needs_newline: false,
            list_stack: vec![],
            #[cfg(feature = "highlight-code")]
            code_highlighter: None,
            styles,
        }
    }

    fn handle_event(&mut self, event: Event<'_>) {
        match event {
            Event::Start(tag) => self.start_tag(tag),
            Event::End(tag) => self.end_tag(tag),
            Event::Text(text) => self.add_text(text),
            Event::Code(code) => self.add_inline_code(code),
            Event::SoftBreak => self.soft_break(),
            Event::HardBreak => self.hard_break(),
            Event::Html(_) | Event::InlineHtml(_) => {
                // Skip HTML for now
            }
            Event::FootnoteReference(_) => {
                // Skip footnotes for now
            }
            Event::Rule => {
                self.add_horizontal_rule();
            }
            _ => {
                // Skip other events for now
            }
        }
    }

    fn start_tag(&mut self, tag: Tag<'_>) {
        match tag {
            Tag::Paragraph => self.start_paragraph(),
            Tag::Heading { level, .. } => self.start_heading(level),
            Tag::BlockQuote(_kind) => self.start_blockquote(),
            Tag::CodeBlock(kind) => self.start_code_block(kind),
            Tag::List(start_index) => self.start_list(start_index),
            Tag::Item => self.start_list_item(),
            Tag::Emphasis => self.push_style(
                parse_style(&self.styles.magenta_accent).add_modifier(Modifier::ITALIC),
            ),
            Tag::Strong => self.push_style(parse_style(&self.styles.bold_text)),
            Tag::Strikethrough => {
                self.push_style(Style::default().add_modifier(Modifier::CROSSED_OUT))
            }
            Tag::Link { .. } => {
                // Store link for later - we could enhance this
                self.push_style(
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::UNDERLINED),
                );
            }
            _ => {
                // Skip unsupported tags for now
            }
        }
    }

    fn end_tag(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Paragraph => self.end_paragraph(),
            TagEnd::Heading(_) => self.end_heading(),
            TagEnd::BlockQuote(_) => self.end_blockquote(),
            TagEnd::CodeBlock => self.end_code_block(),
            TagEnd::List(_) => self.end_list(),
            TagEnd::Item => self.end_list_item(),
            TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough | TagEnd::Link => {
                self.pop_style();
            }
            _ => {
                // Skip unsupported tags
            }
        }
    }

    fn start_paragraph(&mut self) {
        if self.needs_newline {
            self.add_empty_line();
        }
        self.needs_newline = false;
    }

    fn end_paragraph(&mut self) {
        self.finish_current_line();
        self.needs_newline = true;
    }

    fn start_heading(&mut self, level: HeadingLevel) {
        if self.needs_newline {
            self.add_empty_line();
        }

        // Add the heading markers (# ## ###)
        let marker = "#".repeat(level as usize);
        let marker_style = match level {
            HeadingLevel::H1 => parse_style(&self.styles.bold_text).fg(Color::Red),
            HeadingLevel::H2 => parse_style(&self.styles.bold_text).fg(Color::Blue),
            HeadingLevel::H3 => parse_style(&self.styles.bold_text).fg(Color::Green),
            _ => parse_style(&self.styles.bold_text).fg(Color::Cyan),
        };

        self.current_line
            .spans
            .push(Span::styled(marker, marker_style));
        self.current_line.spans.push(Span::raw(" ".to_string()));

        // Set the style for heading text
        self.push_style(marker_style);
        self.needs_newline = false;
    }

    fn end_heading(&mut self) {
        self.pop_style();
        self.finish_current_line();
        self.needs_newline = true;
    }

    fn start_blockquote(&mut self) {
        if self.needs_newline {
            self.add_empty_line();
            self.needs_newline = false;
        }
        self.line_prefixes.push(Span::styled(
            ">".to_string(),
            parse_style(&self.styles.dim_text),
        ));
    }

    fn end_blockquote(&mut self) {
        self.line_prefixes.pop();
        self.needs_newline = true;
    }

    fn start_code_block(&mut self, kind: CodeBlockKind<'_>) {
        if !self.text.lines.is_empty() {
            self.add_empty_line();
        }

        let lang = match kind {
            CodeBlockKind::Fenced(ref lang) => lang.as_ref(),
            CodeBlockKind::Indented => "",
        };

        #[cfg(feature = "highlight-code")]
        self.setup_code_highlighter(lang);

        // NOTE: We DON'T add fence markers here - this is the key fix!
        // The original tui-markdown incorrectly added "```lang" as visible text

        self.needs_newline = false;
    }

    fn end_code_block(&mut self) {
        // NOTE: We DON'T add closing fence markers here either!
        #[cfg(feature = "highlight-code")]
        self.clear_code_highlighter();

        self.finish_current_line();
        self.needs_newline = true;
    }

    #[cfg(feature = "highlight-code")]
    fn setup_code_highlighter(&mut self, lang: &str) {
        if let Some(syntax) = SYNTAX_SET.find_syntax_by_token(lang) {
            let theme = &THEME_SET.themes["base16-ocean.dark"];
            let highlighter = HighlightLines::new(syntax, theme);
            self.code_highlighter = Some(highlighter);
        }
    }

    #[cfg(feature = "highlight-code")]
    fn clear_code_highlighter(&mut self) {
        self.code_highlighter = None;
    }

    fn start_list(&mut self, start_index: Option<u64>) {
        if self.list_stack.is_empty() && self.needs_newline {
            self.add_empty_line();
        }
        self.list_stack.push(start_index);
    }

    fn end_list(&mut self) {
        self.list_stack.pop();
        self.needs_newline = true;
    }

    fn start_list_item(&mut self) {
        self.finish_current_line();

        let indent_level = self.list_stack.len();
        let indent = "  ".repeat(indent_level.saturating_sub(1));

        if let Some(last_index) = self.list_stack.last_mut() {
            let marker = match last_index {
                None => "• ".to_string(), // Unordered list
                Some(index) => {
                    *index += 1;
                    format!("{}. ", *index - 1) // Ordered list
                }
            };

            self.current_line.spans.push(Span::raw(indent));
            self.current_line
                .spans
                .push(Span::styled(marker, parse_style(&self.styles.dim_text)));
        }

        self.needs_newline = false;
    }

    fn end_list_item(&mut self) {
        // List items end naturally when the next item starts or list ends
    }

    fn add_text(&mut self, text: CowStr<'_>) {
        #[cfg(feature = "highlight-code")]
        if let Some(highlighter) = &mut self.code_highlighter {
            // Apply syntax highlighting
            for line in LinesWithEndings::from(&text) {
                if let Ok(highlighted) = highlighter.highlight_line(line, &SYNTAX_SET) {
                    let ansi_string = as_24_bit_terminal_escaped(&highlighted, false);
                    if let Ok(styled_text) = ansi_string.into_text() {
                        for styled_line in styled_text.lines {
                            self.text.lines.push(styled_line);
                        }
                        continue;
                    }
                }
                // Fallback if highlighting fails
                self.current_line.spans.push(Span::styled(
                    line.to_string(),
                    parse_style(&self.styles.dim_text),
                ));
            }
            return;
        }

        // Regular text handling (split by lines)
        let lines: Vec<&str> = text.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if i > 0 {
                self.soft_break();
            }

            let style = self.current_style();
            self.current_line
                .spans
                .push(Span::styled(line.to_string(), style));
        }
    }

    fn add_inline_code(&mut self, code: CowStr<'_>) {
        let code_style = Style::default().fg(Color::White).bg(Color::Black);
        self.current_line
            .spans
            .push(Span::styled(code.to_string(), code_style));
    }

    fn add_horizontal_rule(&mut self) {
        if self.needs_newline {
            self.add_empty_line();
        }

        let rule = "─".repeat(40); // Horizontal line
        self.current_line
            .spans
            .push(Span::styled(rule, parse_style(&self.styles.dim_text)));
        self.finish_current_line();
        self.needs_newline = true;
    }

    fn soft_break(&mut self) {
        self.finish_current_line();
    }

    fn hard_break(&mut self) {
        self.finish_current_line();
    }

    fn push_style(&mut self, style: Style) {
        let current = self.current_style();
        let new_style = current.patch(style);
        self.style_stack.push(new_style);
    }

    fn pop_style(&mut self) {
        if self.style_stack.len() > 1 {
            self.style_stack.pop();
        }
    }

    fn current_style(&self) -> Style {
        *self.style_stack.last().unwrap_or(&Style::default())
    }

    fn finish_current_line(&mut self) {
        // Add line prefixes (for blockquotes, etc.)
        if !self.line_prefixes.is_empty() {
            let mut prefixed_line = Line::default();
            for prefix in &self.line_prefixes {
                prefixed_line.spans.push(prefix.clone());
                prefixed_line.spans.push(Span::raw(" "));
            }
            prefixed_line
                .spans
                .extend(self.current_line.spans.drain(..));
            self.current_line = prefixed_line;
        }

        let line = std::mem::take(&mut self.current_line);
        self.text.lines.push(line);
    }

    fn add_empty_line(&mut self) {
        self.text.lines.push(Line::default());
    }

    fn finish(mut self) -> Text<'static> {
        if !self.current_line.spans.is_empty() {
            self.finish_current_line();
        }

        self.text
    }
}

/// Render markdown text with configurable styling - replacement for tui_markdown::from_str
pub fn render_markdown(markdown: &str, styles: &Styles) -> Text<'static> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(markdown, options);

    let mut renderer = MarkdownRenderer::new(styles);

    for event in parser {
        renderer.handle_event(event);
    }

    renderer.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_core::config_types::Styles;

    #[test]
    fn test_no_fence_markers() {
        let styles = Styles::default();
        let markdown = r#"```rust
fn main() {
    println!("Hello, world!");
}
```"#;

        let result = render_markdown(markdown, &styles);

        // Ensure no fence markers are visible
        for line in &result.lines {
            let content: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
            assert!(
                !content.trim().starts_with("```"),
                "Found fence marker in line: '{}'",
                content
            );
        }

        // Should have the code content
        let all_content: String = result
            .lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|s| s.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");

        assert!(all_content.contains("fn main()"));
        assert!(all_content.contains("println!"));
    }

    #[test]
    fn test_basic_formatting() {
        let styles = Styles::default();
        let markdown = "**Bold** and *italic* text";

        let result = render_markdown(markdown, &styles);
        assert_eq!(result.lines.len(), 1);

        let line = &result.lines[0];
        assert_eq!(line.spans.len(), 4); // "Bold", " and ", "italic", " text"

        // Check that bold span has bold modifier
        assert!(line.spans[0].style.add_modifier.contains(Modifier::BOLD));

        // Check that italic span has italic modifier
        assert!(line.spans[2].style.add_modifier.contains(Modifier::ITALIC));
    }

    #[test]
    fn test_headings() {
        let styles = Styles::default();
        let markdown = "# Heading 1\n## Heading 2";

        let result = render_markdown(markdown, &styles);

        // Should have heading content without fence markers
        let all_content: String = result
            .lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|s| s.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");

        assert!(all_content.contains("# Heading 1"));
        assert!(all_content.contains("## Heading 2"));
    }

    #[test]
    fn test_paragraph_spacing() {
        let styles = Styles::default();
        let markdown = "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.";

        let result = render_markdown(markdown, &styles);

        println!("Total lines: {}", result.lines.len());
        for (i, line) in result.lines.iter().enumerate() {
            let content: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
            println!("Line {}: '{}'", i, content);
        }

        // Should have content with proper spacing
        assert!(result.lines.len() >= 3); // At least the 3 paragraphs
    }

    #[test]
    fn test_styling_application() {
        let styles = Styles::default();
        let markdown = "**Bold text** and *italic text*";

        let result = render_markdown(markdown, &styles);

        println!("Line count: {}", result.lines.len());
        if !result.lines.is_empty() {
            let line = &result.lines[0];
            println!("Span count: {}", line.spans.len());
            for (i, span) in line.spans.iter().enumerate() {
                println!(
                    "Span {}: '{}' - Bold: {:?}, Italic: {:?}",
                    i,
                    span.content,
                    span.style
                        .add_modifier
                        .contains(ratatui::style::Modifier::BOLD),
                    span.style
                        .add_modifier
                        .contains(ratatui::style::Modifier::ITALIC)
                );
            }
        }

        assert_eq!(result.lines.len(), 1);
    }
}
