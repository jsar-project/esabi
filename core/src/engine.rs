#[cfg(feature = "engine-quickjs")]
#[allow(dead_code)]
pub(crate) const SELECTED_ENGINE: &str = "quickjs";

#[cfg(feature = "engine-boa")]
#[allow(dead_code)]
pub(crate) const SELECTED_ENGINE: &str = "boa";
