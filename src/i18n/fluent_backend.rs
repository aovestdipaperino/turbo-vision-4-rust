//! The Fluent backend: one `FluentBundle` per locale of the lookup chain.

use super::Arg;
use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use std::rc::Rc;
use unic_langid::LanguageIdentifier;

/// Every resource added, and the bundles of the current chain.
#[derive(Default)]
pub(crate) struct Store {
    /// (locale, resource), in the order they were added.
    resources: Vec<(String, Rc<FluentResource>)>,
    /// One bundle per locale of the chain, best first.
    bundles: Vec<FluentBundle<Rc<FluentResource>>>,
}

impl Store {
    pub(crate) fn add(&mut self, locale: &str, source: &str) -> Result<(), Vec<String>> {
        let resource = FluentResource::try_new(source.to_string()).map_err(|(_, errors)| {
            errors
                .iter()
                .map(|error| format!("{:?} at {:?}", error.kind, error.pos))
                .collect::<Vec<_>>()
        })?;
        self.resources.push((locale.to_string(), Rc::new(resource)));
        Ok(())
    }

    pub(crate) fn locales(&self) -> Vec<String> {
        let mut locales: Vec<String> = Vec::new();
        for (locale, _) in &self.resources {
            if !locales.contains(locale) {
                locales.push(locale.clone());
            }
        }
        locales
    }

    pub(crate) fn select(&mut self, chain: &[String]) {
        self.bundles = chain
            .iter()
            .filter_map(|locale| {
                let id: LanguageIdentifier = locale.parse().ok()?;
                let mut bundle = FluentBundle::new(vec![id]);
                // Fluent wraps each placeable in U+2068/U+2069 for
                // right-to-left text; a terminal shows them as junk.
                bundle.set_use_isolating(false);
                for (_, resource) in self.resources.iter().filter(|(l, _)| l == locale) {
                    // Later resources replace earlier messages of the same id.
                    bundle.add_resource_overriding(Rc::clone(resource));
                }
                Some(bundle)
            })
            .collect();
    }

    pub(crate) fn has(&self, id: &str) -> bool {
        self.bundles.iter().any(|bundle| {
            bundle
                .get_message(id)
                .is_some_and(|message| message.value().is_some())
        })
    }

    pub(crate) fn format(&self, id: &str, args: &[(&str, Arg<'_>)]) -> Option<String> {
        let fluent_args = (!args.is_empty()).then(|| {
            let mut fluent_args = FluentArgs::new();
            for (name, value) in args {
                let value = match value {
                    Arg::Text(text) => FluentValue::from(text.as_ref()),
                    Arg::Int(n) => FluentValue::from(*n),
                    Arg::Float(n) => FluentValue::from(*n),
                };
                fluent_args.set(*name, value);
            }
            fluent_args
        });
        self.bundles.iter().find_map(|bundle| {
            let pattern = bundle.get_message(id)?.value()?;
            let mut errors = Vec::new();
            let text = bundle.format_pattern(pattern, fluent_args.as_ref(), &mut errors);
            if !errors.is_empty() {
                log::warn!("i18n: '{id}': {errors:?}");
            }
            Some(text.into_owned())
        })
    }
}
