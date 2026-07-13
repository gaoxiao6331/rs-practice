//! # auto-lifetime
//!
//! A small procedural macro that solves the classic *"missing lifetime specifier"*
//! error (`E0106`) for data types that hold references but were written without an
//! explicit lifetime.
//!
//! Apply `#[auto_lifetime]` to a `struct` or `enum`. The macro:
//!
//! 1. adds **one** lifetime parameter (named `'a` by default) to the type, and
//! 2. rewrites every bare reference (`&T`, `&mut T`, `&str`, and references hidden
//!    inside `Vec<&str>`, `Option<&str>`, tuples, etc.) so that it is tagged with
//!    that shared lifetime.
//!
//! ```ignore
//! use auto_lifetime::auto_lifetime;
//!
//! #[auto_lifetime]
//! pub enum LineType {
//!     Heading(&str),
//!     Text(&str),
//! }
//! // expands to:
//! // pub enum LineType<'a> {
//! //     Heading(&'a str),
//! //     Text(&'a str),
//! // }
//! ```
//!
//! ## Behaviour notes
//!
//! * If the type already declares a lifetime parameter, the macro **reuses the first
//!   one** instead of adding a new `'a` (so `struct Foo<'b> { s: &str }` becomes
//!   `struct Foo<'b> { s: &'b str }` rather than gaining a second lifetime).
//! * References that already carry an explicit lifetime are left untouched — the
//!   macro only fills in the *missing* ones. This keeps intentionally multi-lifetime
//!   types correct.
//! * If the type contains no bare references at all, it is emitted unchanged.

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{
    parse_macro_input, Data, DeriveInput, GenericParam, Generics, Lifetime, LifetimeParam,
    TypeReference,
    visit::{self, Visit},
    visit_mut::{self, VisitMut},
};

/// Attribute macro: automatically annotate all reference types with a shared lifetime.
///
/// Works on `struct`, `enum`, and `union`.
#[proc_macro_attribute]
pub fn auto_lifetime(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let expanded = expand_auto_lifetime(input);
    TokenStream::from(quote!(#expanded))
}

/// Core transformation. Adds a shared lifetime and tags every bare reference with it.
fn expand_auto_lifetime(mut input: DeriveInput) -> DeriveInput {
    // 1. Nothing to do if there is no bare reference anywhere.
    if !contains_bare_reference(&input.data) {
        return input;
    }

    // 2. Resolve (or create) the lifetime we will use for every bare reference.
    let lifetime = if let Some(existing) = input.generics.lifetimes().next() {
        // Reuse the first declared lifetime rather than introducing a new one.
        existing.lifetime.clone()
    } else {
        let name = choose_lifetime_name(&input.generics);
        let lt = Lifetime {
            apostrophe: Span::call_site(),
            ident: Ident::new(&name, Span::call_site()),
        };
        input
            .generics
            .params
            .push(GenericParam::Lifetime(LifetimeParam::new(lt.clone())));
        lt
    };

    // 3. Tag every bare reference with that lifetime.
    let mut adder = LifetimeAdder {
        lifetime: &lifetime,
    };
    match &mut input.data {
        Data::Struct(s) => adder.visit_fields_mut(&mut s.fields),
        Data::Enum(e) => {
            for variant in &mut e.variants {
                adder.visit_fields_mut(&mut variant.fields);
            }
        }
        Data::Union(u) => adder.visit_fields_named_mut(&mut u.fields),
    }

    input
}

/// Pick a lifetime name that does not collide with names already in `generics`.
fn choose_lifetime_name(generics: &Generics) -> String {
    let taken: std::collections::HashSet<String> = generics
        .lifetimes()
        .map(|lt| lt.lifetime.ident.to_string())
        .collect();

    for candidate in ["a", "auto", "__auto", "item"] {
        if !taken.contains(candidate) {
            return candidate.to_string();
        }
    }

    let mut i = 0;
    loop {
        let candidate = format!("auto{i}");
        if !taken.contains(&candidate) {
            return candidate;
        }
        i += 1;
    }
}

/// Immutable pass: report whether any reference is missing a lifetime.
struct RefFinder {
    found: bool,
}

impl<'ast> Visit<'ast> for RefFinder {
    fn visit_type_reference(&mut self, node: &'ast TypeReference) {
        if node.lifetime.is_none() {
            self.found = true;
        }
        visit::visit_type_reference(self, node);
    }
}

/// Mutable pass: attach the shared lifetime to every bare reference.
struct LifetimeAdder<'b> {
    lifetime: &'b Lifetime,
}

impl<'ast, 'b> VisitMut for LifetimeAdder<'b> {
    fn visit_type_reference_mut(&mut self, node: &mut TypeReference) {
        if node.lifetime.is_none() {
            node.lifetime = Some(self.lifetime.clone());
        }
        visit_mut::visit_type_reference_mut(self, node);
    }
}

/// Does the type contain any reference without an explicit lifetime?
fn contains_bare_reference(data: &Data) -> bool {
    let mut finder = RefFinder { found: false };
    match data {
        Data::Struct(s) => finder.visit_fields(&s.fields),
        Data::Enum(e) => {
            for variant in &e.variants {
                finder.visit_fields(&variant.fields);
            }
        }
        Data::Union(u) => finder.visit_fields_named(&u.fields),
    }
    finder.found
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    fn expand(src: &str) -> String {
        let input: DeriveInput = syn::parse_str(src).expect("valid item");
        let out = expand_auto_lifetime(input);
        quote!(#out).to_string()
    }

    #[test]
    fn adds_lifetime_to_enum_with_str_refs() {
        let out = expand("pub enum LineType { Heading(&str), Text(&str) }");
        assert!(out.contains("LineType < 'a >"), "got: {out}");
        assert!(out.contains("& 'a str"), "got: {out}");
    }

    #[test]
    fn tags_references_inside_generics() {
        let out = expand("struct Foo { items: Vec<&str>, opt: Option<&str> }");
        assert!(out.contains("Foo < 'a >"), "got: {out}");
        assert!(out.contains("Vec < & 'a str >"), "got: {out}");
        assert!(out.contains("Option < & 'a str >"), "got: {out}");
    }

    #[test]
    fn leaves_non_reference_fields_untouched() {
        let out = expand("struct Foo { n: u64, s: String }");
        // No bare references -> no lifetime parameter added.
        assert!(!out.contains("< 'a >"), "got: {out}");
    }

    #[test]
    fn reuses_existing_lifetime() {
        let out = expand("struct Foo<'b> { s: &str }");
        assert!(out.contains("Foo < 'b >"), "got: {out}");
        assert!(out.contains("& 'b str"), "got: {out}");
        // Should NOT introduce a second lifetime named 'a.
        assert!(!out.contains("< 'a "), "got: {out}");
    }

    #[test]
    fn keeps_explicit_lifetimes() {
        let out = expand("struct Foo<'a> { own: &'a str, borrowed: &str }");
        assert!(out.contains("& 'a str"), "got: {out}");
    }
}
