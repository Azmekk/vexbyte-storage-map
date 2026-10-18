/// Linux never shows a dialog for missing media, so there is nothing to suppress. The type
/// exists so that callers look the same on every platform.
pub(crate) struct MediaErrorPromptSuppression;

impl MediaErrorPromptSuppression {
    pub(crate) fn for_current_thread() -> Self {
        Self
    }
}
