//! mdbook preprocessor that inserts automatically numbered footnotes.
//!
//! Footnotes are included like this: `Normal text{{footnote: Or is it?}} in body.`
//!
//! The `markdown` boolean config value indicates that MarkDown should be emitted for
//! the generated footnotes, rather than HTML.
use clap::{App, Arg, SubCommand};
use lazy_static::lazy_static;
use log::warn;
use mdbook::{
    book::Book,
    errors::Error,
    preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext},
};
use regex::Regex;
use std::collections::HashSet;
use std::{io, process};

/// Name of this preprocessor.
const NAME: &str = "footnote-preprocessor";

pub fn make_app() -> App<'static, 'static> {
    App::new("footnote-preprocessor")
        .about("An mdbook preprocessor which converts expands footnote markers")
        .subcommand(
            SubCommand::with_name("supports")
                .arg(Arg::with_name("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
}

fn main() {
    env_logger::init();
    let matches = make_app().get_matches();
    if let Some(sub_args) = matches.subcommand_matches("supports") {
        let renderer = sub_args.value_of("renderer").expect("Required argument");

        // Signal whether the renderer is supported by exiting with 1 or 0.
        if Footnote::supports_renderer(&renderer) {
            process::exit(0);
        } else {
            process::exit(1);
        }
    } else {
        let (ctx, book) = CmdPreprocessor::parse_input(io::stdin()).expect("Failed to parse input");
        let preprocessor = Footnote::new(&ctx);

        let processed_book = preprocessor
            .run(&ctx, book)
            .expect("Failed to process book");
        serde_json::to_writer(io::stdout(), &processed_book).expect("Faild to emit processed book");
    }
}

lazy_static! {
    static ref FOOTNOTE_RE: Regex =
        Regex::new(r"(?s)\{\{footnote:\s*(?P<content>.*?)\}\}").unwrap();

    /// Names of known renderers which deal in HTML output.
    static ref HTML_RENDERERS: HashSet<String> = {
        let mut s = HashSet::new();
        s.insert("html".to_owned());
        s.insert("linkcheck".to_owned());
        s
    };
}

/// A pre-processor that expands {{footnote: ..}} markers.
#[derive(Default)]
pub struct Footnote {
    md_footnotes: bool,
}

impl Footnote {
    fn new(ctx: &PreprocessorContext) -> Self {
        if ctx.mdbook_version != mdbook::MDBOOK_VERSION {
            // We should probably use the `semver` crate to check compatibility
            // here...
            warn!(
                "The {} plugin was built against version {} of mdbook, \
             but we're being called from version {}",
                NAME,
                mdbook::MDBOOK_VERSION,
                ctx.mdbook_version
            );
        }
        let md_footnotes = if let Some(toml::Value::Boolean(markdown)) =
            ctx.config.get("preprocessor.footnote.markdown")
        {
            *markdown
        } else {
            false
        };

        if !md_footnotes && !HTML_RENDERERS.contains(&ctx.renderer) {
            warn!(
                "Emitting HTML footnotes for renderer '{}' which may not be HTML-based",
                ctx.renderer,
            );
        }

        Self { md_footnotes }
    }

    /// Indicate whether a renderer is supported.  This preprocessor can emit MarkDown so should support almost any
    /// renderer.
    fn supports_renderer(renderer: &str) -> bool {
        renderer != "not-supported"
    }
}

impl Preprocessor for Footnote {
    fn name(&self) -> &str {
        NAME
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        book.for_each_mut(|item| {
            if let mdbook::book::BookItem::Chapter(chap) = item {
                let mut footnotes = vec![];
                chap.content = FOOTNOTE_RE
                    .replace_all(&chap.content, |caps: &regex::Captures| {
                        let content = caps.name("content").unwrap().as_str().to_owned();
                        footnotes.push(content);
                        let idx = footnotes.len();
                        if self.md_footnotes {
                            format!("[^{}]", idx)
                        } else {
                            format!(
                                "<sup><a name=\"to-footnote-{}\">[{}](#footnote-{})</a></sup>",
                                idx, idx, idx
                            )
                        }
                    })
                    .to_string();

                if !footnotes.is_empty() {
                    if self.md_footnotes {
                        chap.content += "<p><hr/>\n";
                    } else {
                        chap.content += "\n---\n";
                    }
                    for (idx, content) in footnotes.into_iter().enumerate() {
                        if self.md_footnotes {
                            chap.content += &format!("\n\n[^{}]: {}", idx + 1, content);
                        } else {
                            chap.content += &format!(
                                "\n\n<a name=\"footnote-{}\">[{}](#to-footnote-{})</a>: {}",
                                idx + 1,
                                idx + 1,
                                idx + 1,
                                content
                            );
                        }
                    }
                }
            }
        });
        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        Self::supports_renderer(renderer)
    }
}
