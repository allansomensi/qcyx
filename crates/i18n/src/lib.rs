use i18n_embed::{
    DefaultLocalizer, LanguageLoader, Localizer,
    fluent::{FluentLanguageLoader, fluent_language_loader},
};
pub use i18n_embed_fl;
use rust_embed::RustEmbed;
use std::sync::LazyLock;

#[derive(RustEmbed)]
#[folder = "../../i18n/"]
struct Localizations;

pub static LANGUAGE_LOADER: LazyLock<FluentLanguageLoader> = LazyLock::new(|| {
    let loader: FluentLanguageLoader = fluent_language_loader!();

    loader
        .load_fallback_language(&Localizations)
        .expect("Error loading fallback language");

    loader
});

#[macro_export]
macro_rules! fl {
    ($message_id:literal) => {{
        $crate::i18n_embed_fl::fl!($crate::LANGUAGE_LOADER, $message_id)
    }};

    ($message_id:literal, $($args:tt)*) => {{
        $crate::i18n_embed_fl::fl!($crate::LANGUAGE_LOADER, $message_id, $($args)*)
    }};
}

pub fn localizer() -> Box<dyn Localizer> {
    Box::from(DefaultLocalizer::new(&*LANGUAGE_LOADER, &Localizations))
}

pub fn localize() {
    let localizer = localizer();
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();

    if let Err(error) = localizer.select(&requested_languages) {
        eprintln!("Error selecting system language: {error}");
    }
}

/// The currently active language, as a Fluent language id (e.g. `"en"`,
/// `"pt-BR"`). Reflects whatever the last [`localize`]/[`set_language`]
/// call selected.
pub fn current_language() -> String {
    LANGUAGE_LOADER.current_language().to_string()
}

/// Languages QCYx ships translations for, as Fluent language ids.
pub const AVAILABLE_LANGUAGES: &[&str] = &["en", "pt-BR"];

/// Switches the active UI language at runtime. Can be called again at any
/// point (e.g. from a settings picker), not just at startup — every
/// `fl!()` call after this returns reflects the new language.
pub fn set_language(language_id: &str) -> Result<(), String> {
    let requested: unic_langid::LanguageIdentifier =
        language_id.parse().map_err(|e| format!("{e}"))?;

    localizer()
        .select(&[requested])
        .map(|_| ())
        .map_err(|e| e.to_string())
}
