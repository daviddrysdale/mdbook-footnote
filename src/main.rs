//! mdbook preprocessor that inserts automatically numbered footnotes.
//!
//! Footnotes are included like this: `Normal text{{footnote: Or is it?}} in body.`
//!
//! The `markdown` boolean config value indicates that MarkDown should be emitted for
//! the generated footnotes, rather than HTML.
use clap::{App, Arg, ArgMatches, SubCommand};
use lazy_static::lazy_static;
use mdbook::{
    book::Book,
    errors::Error,
    preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext},
};
use regex::Regex;
use std::{io, process};

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
    let matches = make_app().get_matches();
    let preprocessor = Footnote::default();

    if let Some(sub_args) = matches.subcommand_matches("supports") {
        handle_supports(&preprocessor, sub_args);
    } else if let Err(e) = handle_preprocessing(preprocessor) {
        eprintln!("{}", e);
        process::exit(1);
    }
}

fn handle_preprocessing(mut pre: Footnote) -> Result<(), Error> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    if ctx.mdbook_version != mdbook::MDBOOK_VERSION {
        // We should probably use the `semver` crate to check compatibility
        // here...
        eprintln!(
            "Warning: The {} plugin was built against version {} of mdbook, \
             but we're being called from version {}",
            pre.name(),
            mdbook::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    pre.md_footnotes = if let Some(toml::Value::Boolean(markdown)) =
        ctx.config.get("preprocessor.footnote.markdown")
    {
        *markdown
    } else {
        false
    };

    let processed_book = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}

fn handle_supports(pre: &dyn Preprocessor, sub_args: &ArgMatches) -> ! {
    let renderer = sub_args.value_of("renderer").expect("Required argument");
    let supported = pre.supports_renderer(&renderer);

    // Signal whether the renderer is supported by exiting with 1 or 0.
    if supported {
        process::exit(0);
    } else {
        process::exit(1);
    }
}

lazy_static! {
    static ref FOOTNOTE_RE: Regex =
        Regex::new(r"(?s)\{\{footnote:\s*(?P<content>.*?)\}\}").unwrap();
}

/// A pre-processor that expands {{footnote: ..}} markers.
#[derive(Default)]
pub struct Footnote {
    md_footnotes: bool,
}

impl Preprocessor for Footnote {
    fn name(&self) -> &str {
        "footnote-preprocessor"
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
        renderer != "not-supported"
    }
}
