//! Generates highlighted HTML with CSS classes for a Rust, using syntect and markdown
//! Run with ```cargo run --bin make_html```
use std::fs::read_to_string;
use pulldown_cmark::Parser;
use syntect::highlighting::ThemeSet;
use syntect::html::css_for_theme_with_class_style;
use syntect::html::{ClassStyle, ClassedHTMLGenerator};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn output_code_block(ss: &SyntaxSet, mut html: impl Write, src_text: &str) -> Result<(), std::io::Error> {
    let sr_rs = ss.find_syntax_by_extension("rs").unwrap();
    let mut rs_html_generator =
        ClassedHTMLGenerator::new_with_class_style(sr_rs, &ss, ClassStyle::Spaced);
    for line in LinesWithEndings::from(src_text) {
        rs_html_generator
            .parse_html_for_line_which_includes_newline(line)
            .unwrap();
    }
    let html_rs = rs_html_generator.finalize();

    writeln!(html, "<div class=\"code_block\"><pre class=\"code\">")?;
    writeln!(html, "{}", html_rs)?;
    writeln!(html, "</pre></div>")?;

    Ok(())
}

fn output_doc_block(mut html: impl Write, doc_text: &str) -> Result<(), std::io::Error> {
    // markdown
    let parser = Parser::new(doc_text);

    // Write to a new String buffer.
    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, parser);
    writeln!(html, "<div class=\"doc_block\">")?;
    writeln!(html, "{}", html_output)?;
    writeln!(html, "</div>")?;

    Ok(())
}

fn output_html(
    ss: &SyntaxSet,
    src_file: &str,
    out_file: &str,
    title: &str,
) -> Result<(), std::io::Error> {
    let html_file = File::create(Path::new(out_file))?;
    let mut html = BufWriter::new(&html_file);

    // write html header
    writeln!(html, "<!DOCTYPE html>")?;
    writeln!(html, "<html>")?;
    writeln!(html, "  <head>")?;
    writeln!(html, "    <title>{}</title>", title)?;

    writeln!(html, "    <style type=\"text/css\">")?;
    writeln!(html, "{}", include_str!("make_html.css"))?;
    writeln!(html, "    </style>")?;

    writeln!(html, "  </head>")?;
    writeln!(html, "  <body>")?;

    // Load code, split into documentation blocks and code blocks
    let mut reading_doc = false;
    let mut strbuf = "".to_string();
    writeln!(html, "  <div class=\"row\">")?;
    writeln!(html, "  <div class=\"doc_group\">")?;
    for line in read_to_string(src_file).unwrap().lines() {
        if line.starts_with("///") {
            if !reading_doc {
                reading_doc = true;
                let code_buf = strbuf.trim_end();
                if code_buf.len() > 0 {
                    writeln!(html, "  </div>")?;
                    output_code_block(&ss, &mut html, &code_buf)?;
                    writeln!(html, "  </div>")?;
                    writeln!(html, "  <div class=\"row\">")?;
                    writeln!(html, "  <div class=\"doc_group\">")?;
                }
                strbuf = "".to_string();
            }
            strbuf += &line[3..];
            strbuf += "\n";
        } else {
            if reading_doc {
                reading_doc = false;
                output_doc_block(&mut html, &strbuf)?;
                strbuf = "".to_string();
            }
            // section dividers
            if line.starts_with("//-") {
                writeln!(html, "  </div>")?;
                writeln!(html, "  </div>")?;
                writeln!(html, "  <div class=\"row\">")?;
                writeln!(html, "  <div class=\"doc_group\">")?;
            // doc comment warning suppression
            } else if !line.starts_with("#[allow(unused_doc_comments)]") {
                strbuf += &(line.to_owned() + "\n");
            }
        }
    }
    // output last buffer
    if reading_doc {
        output_doc_block(&mut html, &strbuf)?;
    } else {
        let code_buf = strbuf.trim_end();
        if code_buf.len() > 0 {
            writeln!(html, "  </div>")?;
            output_code_block(&ss, &mut html, &code_buf)?;
        }
    }
    writeln!(html, "  </div>")?;

    // write html end
    writeln!(html, "  </body>")?;
    writeln!(html, "</html>")?;

    Ok(())
}

fn main() -> Result<(), std::io::Error> {
    // ---------------------------------------------------------------------------------------------
    // generate html
    let ss = SyntaxSet::load_defaults_newlines();

    output_html(&ss, "src/skip_list.rs", "skip_list.html", "Skip List in Rust")?;

    // ---------------------------------------------------------------------------------------------
    // generate css files for themes
    let ts = ThemeSet::load_defaults();

    // create dark color scheme css
    let dark_theme = &ts.themes["base16-eighties.dark"];
    let css_dark_file = File::create(Path::new("theme-dark.css"))?;
    let mut css_dark_writer = BufWriter::new(&css_dark_file);

    let css_dark = css_for_theme_with_class_style(dark_theme, ClassStyle::Spaced).unwrap();
    writeln!(css_dark_writer, "{}", css_dark)?;

    // create light color scheme css
    let light_theme = &ts.themes["Solarized (light)"];
    let css_light_file = File::create(Path::new("theme-light.css"))?;
    let mut css_light_writer = BufWriter::new(&css_light_file);

    let css_light = css_for_theme_with_class_style(light_theme, ClassStyle::Spaced).unwrap();
    writeln!(css_light_writer, "{}", css_light)?;

    Ok(())
}
