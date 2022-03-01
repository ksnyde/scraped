use scraper::ElementRef;
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::trace;
use url::Url;

use crate::element::{Element, HrefSource, HrefType, ImageType, SourceType};

fn foreign_or_local(url: &Url, el: &Element) -> SourceType {
    let domain = url.domain();
    match (domain, &el.full_href) {
        (Some(domain), Some(href)) if href.contains(domain) => SourceType::HtmlSameSite,
        (Some(_domain), Some(_href)) => SourceType::HtmlForeignSite,
        _ => SourceType::Unknown,
    }
}

/// Gets all the name/value pairs available on the passed in DOM
/// node and returns as a serde::Value enumeration
pub fn get_selection(el_ref: ElementRef, url: &Url) -> Element {
    let mut el = Element::new(el_ref.value().name());

    // attrs hashmap
    let mut attrs: HashMap<String, Value> = HashMap::new();
    el_ref.value().attrs().for_each(|(k, v)| {
        if !v.is_empty() {
            attrs.insert(k.to_string(), json!(v));
        }
    });

    // text and html
    let text = el_ref.text().collect::<String>();
    if !text.is_empty() {
        el.text = Some(text.trim().to_string());
    }
    if !el_ref.inner_html().is_empty() {
        el.html = Some(el_ref.inner_html().trim().to_string());
    }

    // _type / type
    if el_ref.value().attr("type").is_some() {
        el._type = Some(el_ref.value().attr("type").unwrap().to_string());
    }

    // meta
    let only_child = (el_ref.has_children()) && (el_ref.first_child() == el_ref.last_child());
    let first_child_href = if let Some(child) = el_ref.first_child() {
        match child.value().as_element() {
            Some(child) => {
                let href = child.attr("href");

                if href.is_some() {
                    href
                } else {
                    None
                }
            }
            None => None,
        }
    } else {
        None
    };

    // href
    if el_ref.value().attr("href").is_some() {
        let href = el_ref.value().attr("href").unwrap();
        el.href = Some(href.to_string());
        el.href_source = Some(HrefSource::Element);
    } else if (only_child) && (first_child_href.is_some()) {
        let href = first_child_href.unwrap();
        el.href = Some(href.to_string());
        el.href_source = Some(HrefSource::OnlyChild);
    }

    if let Some(href) = &el.href {
        let url = url.join(href).expect("full url should be parsable");
        let ext = href.split('.').last();
        // address meta on href
        if href.starts_with("http") {
            el.full_href = Some(href.to_string());
            el.href_type = Some(HrefType::Absolute);
        } else if href.is_empty() {
            el.href_type = Some(HrefType::Empty);
        } else if href.contains("Javascript(") {
            el.href_type = Some(HrefType::Javascript);
        } else if href.trim().eq("#") {
            el.href_type = Some(HrefType::SelfReferencingAnchor);
            el.full_href = Some(format!("{}", url));
        } else if href.starts_with('#') {
            el.href_type = Some(HrefType::AnchorLink);
            el.full_href = Some(format!("{}", url));
        } else {
            el.href_type = Some(HrefType::Relative);
            el.full_href = Some(format!("{}", url));
        }

        if let Some(ext) = ext {
            let source_type = match ext {
                "woff" => SourceType::Font,
                "woff2" => SourceType::Font,
                "ttf" => SourceType::Font,
                "otf" => SourceType::Font,
                "fnt" => SourceType::Font,
                "css" => SourceType::Style,
                "svg" => SourceType::Image,
                "jpg" => SourceType::Image,
                "jpeg" => SourceType::Image,
                "png" => SourceType::Image,
                "ico" => SourceType::Image,
                "html" => match el.href_type {
                    Some(HrefType::Relative) => SourceType::HtmlSameSite,
                    Some(HrefType::AnchorLink) => SourceType::HtmlSameSite,
                    Some(HrefType::SelfReferencingAnchor) => SourceType::HtmlSameSite,
                    Some(HrefType::Javascript) => SourceType::Unknown,
                    Some(HrefType::Empty) => SourceType::Unknown,
                    Some(HrefType::Absolute) => {
                        let domain = url.domain();
                        match (domain, &el.full_href) {
                            (Some(_domain), Some(_href)) => foreign_or_local(&url, &el),
                            _ => SourceType::Unknown,
                        }
                    }
                    _ => SourceType::Unknown,
                },
                _ => {
                    if &el.tag_name == "a" {
                        foreign_or_local(&url, &el)
                    } else {
                        SourceType::Unknown
                    }
                }
            };

            el.target_type = Some(source_type);
        }
    }

    // let image_formats = ["gif", "jpg", "jpeg", "avif", "webp", "ico", "tiff"];
    // TODO: finish off other formats and their meta info
    // let doc_formats = ["doc", "docx", "pdf"];
    // let data_formats = ["csv", "json", "txt", "xls"];
    // let code_formats = ["js", "wasm"];
    // let style_formats = ["css", "scss", "sass"];

    // src
    if el_ref.value().attr("src").is_some() {
        let src = el_ref.value().attr("src").unwrap();
        let ext = src.split('.').last();
        el.src = Some(src.to_string());

        // if we can see a file extension
        if let Some(ext) = ext {
            if el.tag_name == "img" {
                el.target_type = Some(SourceType::Image);
            }
            let image_type = match ext {
                "gif" => Some(ImageType::Gif),
                "jpg" => Some(ImageType::Jpeg),
                "jpeg" => Some(ImageType::Jpeg),
                "avif" => Some(ImageType::Avif),
                "webp" => Some(ImageType::Webp),
                "ico" => Some(ImageType::Ico),
                "png" => Some(ImageType::Png),
                "tiff" => Some(ImageType::Tiff),
                "svg" => Some(ImageType::Svg),
                _ => {
                    if el.tag_name == "img" {
                        Some(ImageType::Other(ext.to_string().to_lowercase()))
                    } else {
                        None
                    }
                }
            };
            el.image_type = image_type;
        }
    }

    el.attrs = attrs;

    trace!("[{}] selection for node completed", url);

    el
}
