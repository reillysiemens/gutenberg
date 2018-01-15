use std::fmt::{self, Write};
use std::borrow::Cow;
use std::collections::HashMap;
use ast::{Content, Node};
use pulldown_cmark::{Event, Tag};

pub trait IntoHtml<C> {
    fn render(&mut self, ctx: &mut C, buf: &mut String);
}

enum TagType {
    Opening,
    Closing,
}

struct Context<'a> {
    tag_type: Option<TagType>,
    footnote_indices: HashMap<Cow<'a, str>, usize>,
}

impl<'a> Context<'a> {
    fn new() -> Context<'a> {
        Context {
            tag_type: None,
            footnote_indices: HashMap::new(),
        }
    }

    fn render_tag(&self, tag: &str, buf: &mut String) {
        let tag_closer = match self.tag_type {
           Some(TagType::Closing) => "/",
            _ => "",
        };
        buf.push_str(&format!("<{}{}>", tag_closer, tag));
    }

    fn render_nested_tags(&self, tags: &[&str], buf: &mut String) {
        match self.tag_type {
            Some(TagType::Opening) => {
                tags.into_iter().for_each(|t| self.render_tag(t, buf));
            },
            Some(TagType::Closing) => {
                tags.into_iter().rev().for_each(|t| self.render_tag(t, buf));
            },
            None => (),
        };
    }

    fn get_footnote_index(&mut self, id: Cow<'a, str>) -> usize {
        let num_footnotes = self.footnote_indices.len() + 1;
        *self.footnote_indices.entry(id).or_insert(num_footnotes)
    }

    fn render_footnote_reference(&mut self, id: Cow<'a, str>, buf: &mut String) {
        buf.push_str("<sup class=\"footnote-reference\"><a href=\"#");
        // We unwrap here because the String writer implementation will never
        // fail.
        escape_html(buf, &id).unwrap();
        buf.push_str("\">");
        buf.push_str(&*format!("{}", self.get_footnote_index(id)));
        buf.push_str("</a></sup>");
    }

    fn render_footnote_definition(&mut self, id: Cow<'a, str>, buf: &mut String) {
        match self.tag_type {
            Some(TagType::Opening) => {
                buf.push_str(
                    "<div class=\"footnote-definition\" id=\"",
                );
                // We unwrap here because the String writer implementation will never
                // fail.
                escape_html(buf, &*id).unwrap();
                buf.push_str("\"><sup class=\"footnote-definition-label\">");
                buf.push_str(&*format!("{}", self.get_footnote_index(id)));
                buf.push_str("</sup>\n");
            },
            Some(TagType::Closing) => {
                buf.push_str("</div>")
            },
            None => (),
        }
    }
}

impl<'a> IntoHtml<Context<'a>> for Tag<'a> {
    fn render(&mut self, context: &mut Context<'a>, buf: &mut String) {
        match *self {
            Tag::Paragraph => context.render_tag("p", buf),
            Tag::Header(n) => context.render_tag(&format!("h{}", n), buf),
            Tag::CodeBlock(ref _info_string) => context.render_nested_tags(&["pre", "code"], buf),
            Tag::FootnoteDefinition(ref id) => context.render_footnote_definition(id.clone(), buf),
            _ => (),
        }
    }
}

impl<'a> IntoHtml<Context<'a>> for Event<'a> {
    fn render(&mut self, context: &mut Context<'a>, buf: &mut String) {
        match *self {
            Event::Text(ref text) | Event::Html(ref text) | Event::InlineHtml(ref text) => buf.push_str(text),
            Event::FootnoteReference(ref id) => context.render_footnote_reference(id.clone(), buf),
            Event::Start(_) | Event::End(_) => unreachable!(),
            _ => panic!("AHHHHHHH!!!!!!!!!!"),
        }
    }
}

impl<'a> IntoHtml<Context<'a>> for Node<'a> {
    fn render(&mut self, context: &mut Context<'a>, buf: &mut String) {
        match *self {
            Node::Block(ref mut tag, ref mut content) => {
                context.tag_type = Some(TagType::Opening);
                tag.render(context, buf);

                context.tag_type = None;
                content.render(context, buf);

                context.tag_type = Some(TagType::Closing);
                tag.render(context, buf);
                buf.push('\n');
                context.tag_type = None;
            },
            Node::Item(ref mut event) => event.render(context, buf),
        }

    }
}

impl<'a, I> IntoHtml<Context<'a>> for Content<'a, I>
where
    I: Iterator<Item = Event<'a>>,
{
    fn render(&mut self, context: &mut Context<'a>, buf: &mut String) {
        for mut node in self {
            node.render(context, buf);
        }
    }
}

pub fn into_html<'a, I>(content: &mut Content<'a, I>, buf: &mut String)
where
    I: Iterator<Item = Event<'a>>
{
    let mut context = Context::new();
    content.render(&mut context, buf);
}

fn escape_html<W: Write>(buf: &mut W, html: &str) -> Result<(), fmt::Error> {
    for c in html.as_bytes() {
        match *c {
            b'"' => buf.write_str("&quot;")?,
            b'&' => buf.write_str("&amp;")?,
            b'\'' => buf.write_str("&#47;")?,
            b'<' => buf.write_str("&lt;")?,
            b'>' => buf.write_str("&gt;")?,
            _ => buf.write_char(*c as char)?,
        }
    }
    Ok(())
}
