// TODO: Probably could and should turn this into a separate crate.

use {
    autocfg::AutoCfg,
    std::collections::HashMap,
};


fn emit_cargo_instruction(
    instruction: &str,
    arg: Option<&str>,
)
{
    assert!(!instruction.is_empty());
    if let Some(arg) = arg {
        assert!(!arg.is_empty());
    }
    println!("cargo:{}{}", instruction, arg.map(|s| format!("={}", s)).as_deref().unwrap_or(""));
}

macro_rules! emit_rerun_if_changed_self_only {
    () => {
        emit_cargo_instruction("rerun-if-changed", Some(file!()))
    };
}

fn emit_warning(message: &str)
{
    emit_cargo_instruction("warning", Some(message));
}

/// Pass a key-value configuration option to the compiler to be set for conditional compilation,
/// for features of the Rust compiler, language, or standard library.
///
/// This enables using [the standard conditional-compilation
/// forms](https://doc.rust-lang.org/reference/conditional-compilation.html) (i.e. the `cfg`
/// attribute, et al) for features of Rust itself, in a way that is more similar to Cargo package
/// features.
///
/// `key_category`: One of `"comp"`, `"lang"`, or `"lib"`.
///
/// `value`: The feature name, which should follow [The Unstable
/// Book](https://doc.rust-lang.org/nightly/unstable-book/index.html) where appropriate.
///
/// # Examples
///
/// Doing `emit_rust_feature("lib", "step_trait")` in a package's build script enables the
/// package's source code to use `#[cfg(rust_lib_feature = "step_trait")]`.
///
/// # Panics
///
/// If `key_category` is not one of the acceptable categories.
fn emit_rust_feature(
    key_category: &str,
    value: &str,
)
{
    assert!(["comp", "lang", "lib"].contains(&key_category));
    // TODO: Is `value` being properly quoted for all cases?
    emit_cargo_instruction(
        "rustc-cfg",
        Some(&format!("rust_{}_feature={:?}", key_category, value)),
    )
}

/// TODO: Like [`emit_rust_feature`] but also checks that the feature is not one of the known
/// official ones.
fn emit_extra_rust_feature(
    key_category: &str,
    value: &str,
)
{
    // TODO: Check that not in official features.
    // TODO: Try to clean-up to be concise.
    #[allow(clippy::match_like_matches_macro)]
    let is_valid = match key_category {
        "comp" => match value {
            "unstable_features" => true,
            _ => false,
        },
        _ => false,
    };
    if is_valid {
        emit_rust_feature(key_category, value);
    }
    else {
        todo!("Incomplete list.");
    }
}


/// Custom methods on [`AutoCfg`].
trait CustomAutoCfg
{
    // TODO: How should this be if turned into a separate crate?  It would not provide much, for
    // general users.  Maybe that's ok - just expect support for additional features to be added
    // over time as demanded, and increment the minor version each time.
    /// Tests whether the current `rustc` provides the given compiler/language/library feature as
    /// stable (i.e. without needing the `#![feature(...)]` of nightly).
    ///
    /// `feature`: One of the "feature flags" named by
    /// <https://doc.rust-lang.org/nightly/unstable-book/index.html>.
    fn probe_rust_feature(
        &self,
        feature: &str,
    ) -> Option<&'static str>;

    fn emit_rust_feature(
        &self,
        feature: &str,
    ) -> bool
    {
        if let Some(key_category) = self.probe_rust_feature(feature) {
            emit_rust_feature(key_category, feature);
            true
        }
        else {
            false
        }
    }

    fn emit_rust_features<'l>(
        &self,
        features: impl IntoIterator<Item = &'l str>,
    ) -> HashMap<&'l str, bool>
    {
        use core::iter::repeat;

        let mut features = HashMap::from_iter(features.into_iter().zip(repeat(false)));
        let mut any_stable_rust_feature = false;

        for (feature, is_stable) in features.iter_mut() {
            *is_stable = self.emit_rust_feature(feature);
            any_stable_rust_feature = *is_stable || any_stable_rust_feature;
        }
        if any_stable_rust_feature && self.probe_rust_feature("cfg_version").is_some() {
            emit_warning("Rust feature cfg_version is now stable. Consider using instead.");
        }
        features
    }
}

impl CustomAutoCfg for AutoCfg
{
    fn probe_rust_feature(
        &self,
        feature: &str,
    ) -> Option<&'static str>
    {
        // TODO: Could improve with some static CATEGORY_TABLE: Once that associates feature to
        // category, which would allow factoring-out redundant `const CATEGORY` and redundant
        // `.then(|| ...)`.

        match feature {
            "cfg_version" => {
                const CATEGORY: &str = "lang";
                const EXPR: &str = r#"{ #[cfg(version("1.0"))] struct X; X }"#;
                self.probe_expression(EXPR).then(|| CATEGORY)
            },
            "step_trait" => {
                const CATEGORY: &str = "lib";
                const PATH: &str = "std::iter::Step";
                self.probe_path(PATH).then(|| CATEGORY)
            },
            "unwrap_infallible" => {
                const CATEGORY: &str = "lib";
                const EXPR: &str = r#"Ok::<(), core::convert::Infallible>(()).into_ok()"#;
                self.probe_expression(EXPR).then(|| CATEGORY)
            },
            _ => todo!(),
        }
    }
}


fn main() -> Result<(), &'static str>
{
    emit_rerun_if_changed_self_only!();

    // TODO: the_crate::emit_rust_features(...)
    autocfg::new().emit_rust_features(["step_trait", "unwrap_infallible"]);

    // TODO: the_crate::emit_rust_comp_feature("unstable_features")
    let rust_release_channel = version_check::Channel::read().unwrap();
    if rust_release_channel.supports_features() {
        emit_extra_rust_feature("comp", "unstable_features");
    }

    Ok(())
}
