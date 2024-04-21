use proc_macro2::{Span, TokenTree};
use syn::{Attribute, Ident};

pub(crate) fn concat_idents(ident1: &Ident, ident2: &Ident) -> Ident {
    let combined = format!("{}{}", ident1, ident2);
    Ident::new(&combined, Span::call_site())
}

pub(crate) fn snake_case_to_camel_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut c = word.chars();
            c.next()
                .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
                .unwrap_or_default()
        })
        .collect()
}

pub(crate) fn collect_table_attr(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if !attr.path.is_ident("laser") {
            // ignore non-laser attributes
            continue;
        }

        let mut token_iter = attr
            .tokens
            .clone()
            .into_iter()
            .filter_map(|token_tree| match token_tree {
                TokenTree::Group(group) => Some(group.stream().into_iter()),
                _ => None,
            })
            .flatten();

        while let Some(t) = token_iter.next() {
            if t.to_string() != "table" {
                // ignore non-table attributes
                continue;
            }
            if let Some(t) = token_iter.next() {
                if t.to_string() != "=" {
                    // ignore non-assignment attributes
                    continue;
                }
                if let Some(t) = token_iter.next() {
                    // collect the table name
                    if t.to_string().starts_with('\"') && t.to_string().ends_with('\"') {
                        return Some(t.to_string()[1..t.to_string().len() - 1].to_owned());
                    }
                }
            }
        }
    }

    None
}

pub(crate) fn collect_filter_attrs(attrs: &[Attribute]) -> Vec<String> {
    for attr in attrs {
        if !attr.path.is_ident("laser") {
            // ignore non-laser attributes
            continue;
        }

        return attr
            .tokens
            .clone()
            .into_iter()
            .filter_map(|token_tree| match token_tree {
                TokenTree::Group(group) => Some(group.stream().into_iter()),
                _ => None,
            })
            .flatten()
            .filter_map(|t| {
                let s = t.to_string();
                match s.as_str() {
                    "\"==\"" => Some("==".to_string()),
                    "\"!=\"" => Some("!=".to_string()),
                    "\"<\"" => Some("<".to_string()),
                    "\"<=\"" => Some("<=".to_string()),
                    "\">\"" => Some(">".to_string()),
                    "\">=\"" => Some(">=".to_string()),
                    _ => None,
                }
            })
            .collect();
    }
    vec![]
}

pub(crate) fn has_pk_attr(attrs: &[Attribute]) -> bool {
    has_any_attr(&["pk", "primary_key"], attrs)
}

pub(crate) fn has_flatten_attr(attrs: &[Attribute]) -> bool {
    has_any_attr(&["flat", "flatten"], attrs)
}

pub(crate) fn has_skip_attr(attrs: &[Attribute]) -> bool {
    has_any_attr(&["skip"], attrs)
}

pub(crate) fn has_skip_filter_attr(attrs: &[Attribute]) -> bool {
    has_any_attr(&["skip", "skip_filter"], attrs)
}

pub(crate) fn has_skip_sort_attr(attrs: &[Attribute]) -> bool {
    has_any_attr(&["skip", "skip_sort"], attrs)
}

pub(crate) fn has_json_attr(attrs: &[Attribute]) -> bool {
    has_any_attr(&["json"], attrs)
}

fn has_any_attr(options: &[&str], attrs: &[Attribute]) -> bool {
    for attr in attrs {
        if !attr.path.is_ident("laser") {
            // ignore non-laser attributes
            continue;
        }
        // find any skip or skip_sort attributes
        if attr
            .tokens
            .clone()
            .into_iter()
            .filter_map(|token_tree| match token_tree {
                TokenTree::Group(group) => Some(group.stream().into_iter()),
                _ => None,
            })
            .flatten()
            .any(|t| {
                let s = t.to_string();
                options.iter().any(|o| &s == o)
            })
        {
            return true;
        }
    }
    false
}
