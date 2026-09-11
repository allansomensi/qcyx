//! Built-in profiles store a translation key in their `name` field (see
//! [`qcyx_core::profile::Profile::built_in`]); user-saved profiles store
//! literal text the user typed. This tells the two apart and returns the
//! right thing to show in the UI, since `fl!()` needs a compile-time
//! literal and can't look up a runtime key directly.

use qcyx_core::profile::Profile;

/// The label to display for a profile's name.
///
/// Reserved names can't be saved or imported, so the lookup only ever runs
/// for built-in keys. Allocation-free check: this runs per row, per frame.
pub fn profile_label(name: &str) -> String {
    if Profile::is_reserved_name(name) && qcyx_i18n::LANGUAGE_LOADER.has(name) {
        qcyx_i18n::LANGUAGE_LOADER.get(name)
    } else {
        name.to_string()
    }
}
