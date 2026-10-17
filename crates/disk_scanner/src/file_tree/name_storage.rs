use std::collections::HashMap;

use file_categories::{FileCategory, category_for_extension};

/// Identifies a name stored in `NameStorage`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct NameId(u32);

/// Identifies an interned, lowercase file extension. `ExtensionId::NONE` means no extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct ExtensionId(u32);

impl ExtensionId {
    pub(crate) const NONE: Self = Self(0);

    pub(crate) fn as_usize(self) -> usize {
        self.0 as usize
    }
}

/// Extensions longer than this are treated as part of the name (for example in
/// `report.final-version-2`), so they do not flood the file type breakdown.
const MAXIMUM_EXTENSION_LENGTH: usize = 16;

/// Stores every node name in one large buffer, addressed by offsets, instead of one heap
/// allocation per name. Extensions are interned separately, so a file type breakdown only
/// aggregates a few thousand distinct ids instead of comparing strings.
#[derive(Debug, Clone)]
pub(crate) struct NameStorage {
    name_text: String,
    /// `name_offsets[id]..name_offsets[id + 1]` is the byte range of name `id`.
    name_offsets: Vec<u32>,
    extensions: Vec<Box<str>>,
    extension_categories: Vec<Option<FileCategory>>,
    extension_ids_by_text: HashMap<Box<str>, ExtensionId>,
}

impl NameStorage {
    pub(crate) fn with_capacity(name_count: usize) -> Self {
        let mut name_offsets = Vec::with_capacity(name_count + 1);
        name_offsets.push(0);

        Self {
            name_text: String::with_capacity(name_count * 16),
            name_offsets,
            extensions: vec![Box::from("")],
            extension_categories: vec![None],
            extension_ids_by_text: HashMap::new(),
        }
    }

    pub(crate) fn add_name(&mut self, name: &str) -> NameId {
        let id = NameId(u32::try_from(self.name_offsets.len() - 1).unwrap_or(u32::MAX));
        self.name_text.push_str(name);
        self.name_offsets
            .push(u32::try_from(self.name_text.len()).unwrap_or(u32::MAX));
        id
    }

    /// The same names in the order `ids` gives, with the same extension ids: name `i` of the
    /// result is name `ids[i]` of `self`. The text is allocated at its exact size.
    pub(crate) fn into_reordered(self, ids: impl Iterator<Item = NameId> + Clone) -> Self {
        let text_length: usize = ids.clone().map(|id| self.name(id).len()).sum();
        let name_count = ids.clone().count();
        let mut name_text = String::with_capacity(text_length);
        let mut name_offsets = Vec::with_capacity(name_count + 1);
        name_offsets.push(0);
        for id in ids {
            name_text.push_str(self.name(id));
            name_offsets.push(u32::try_from(name_text.len()).unwrap_or(u32::MAX));
        }

        Self {
            name_text,
            name_offsets,
            ..self
        }
    }

    /// The name stored at `position`, for storages whose names are in node order.
    pub(crate) fn name_at(&self, position: usize) -> &str {
        self.name(NameId(u32::try_from(position).unwrap_or(u32::MAX)))
    }

    /// The number of names stored.
    #[cfg(test)]
    pub(crate) fn name_count(&self) -> usize {
        self.name_offsets.len() - 1
    }

    /// Approximate heap memory: the name text, its offsets and the interned extensions.
    pub(crate) fn memory_bytes(&self) -> usize {
        let extension_text_bytes: usize = self.extensions.iter().map(|text| text.len()).sum();
        let per_extension_bytes = 2 * size_of::<Box<str>>()
            + size_of::<ExtensionId>()
            + size_of::<Option<FileCategory>>();
        self.name_text.capacity()
            + self.name_offsets.capacity() * size_of::<u32>()
            + 2 * extension_text_bytes
            + self.extensions.len() * per_extension_bytes
    }

    pub(crate) fn name(&self, id: NameId) -> &str {
        let index = id.0 as usize;
        let (Some(&start), Some(&end)) = (
            self.name_offsets.get(index),
            self.name_offsets.get(index + 1),
        ) else {
            return "";
        };
        self.name_text
            .get(start as usize..end as usize)
            .unwrap_or("")
    }

    /// Interns the lowercase extension of `file_name`, or returns `ExtensionId::NONE`.
    pub(crate) fn intern_extension_of(&mut self, file_name: &str) -> ExtensionId {
        let Some(extension) = extension_of(file_name) else {
            return ExtensionId::NONE;
        };

        let is_already_lowercase = !extension.chars().any(char::is_uppercase);
        if is_already_lowercase {
            if let Some(&id) = self.extension_ids_by_text.get(extension) {
                return id;
            }
            return self.add_extension(extension.into());
        }

        let lowercase_extension = extension.to_lowercase();
        if let Some(&id) = self.extension_ids_by_text.get(lowercase_extension.as_str()) {
            return id;
        }
        self.add_extension(lowercase_extension.into_boxed_str())
    }

    fn add_extension(&mut self, lowercase_extension: Box<str>) -> ExtensionId {
        let id = ExtensionId(u32::try_from(self.extensions.len()).unwrap_or(0));
        self.extension_categories
            .push(category_for_extension(&lowercase_extension));
        self.extensions.push(lowercase_extension.clone());
        self.extension_ids_by_text.insert(lowercase_extension, id);
        id
    }

    /// The lowercase extension without its dot, or `None` for `ExtensionId::NONE`.
    pub(crate) fn extension(&self, id: ExtensionId) -> Option<&str> {
        if id == ExtensionId::NONE {
            return None;
        }
        self.extensions.get(id.as_usize()).map(AsRef::as_ref)
    }

    /// The id of an already interned extension, given in lowercase without its dot.
    pub(crate) fn interned_extension_id(&self, lowercase_extension: &str) -> Option<ExtensionId> {
        self.extension_ids_by_text.get(lowercase_extension).copied()
    }

    pub(crate) fn extension_category(&self, id: ExtensionId) -> Option<FileCategory> {
        self.extension_categories
            .get(id.as_usize())
            .copied()
            .flatten()
    }

    /// The number of extension ids including `ExtensionId::NONE`, for building lookup tables.
    pub(crate) fn extension_id_count(&self) -> usize {
        self.extensions.len()
    }

    pub(crate) fn extension_id_from_usize(index: usize) -> ExtensionId {
        ExtensionId(u32::try_from(index).unwrap_or(0))
    }

    pub(crate) fn name_text_bytes(&self) -> usize {
        self.name_text.len()
    }
}

/// The text after the last dot. Windows treats `.gitignore` as a file with extension
/// `gitignore`, and so does this function.
fn extension_of(file_name: &str) -> Option<&str> {
    let dot_position = file_name.rfind('.')?;
    let extension = &file_name[dot_position + 1..];

    is_plausible_extension(extension).then_some(extension)
}

/// True when `extension` (without its dot) is something `NameStorage` interns as an extension
/// rather than treating it as part of the name.
pub(crate) fn is_plausible_extension(extension: &str) -> bool {
    !extension.is_empty()
        && extension.chars().count() <= MAXIMUM_EXTENSION_LENGTH
        && !extension.contains(' ')
        && !extension.contains('.')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_and_returns_names() {
        let mut storage = NameStorage::with_capacity(4);

        let first = storage.add_name("Photos");
        let second = storage.add_name("");
        let third = storage.add_name("Überweisung.pdf");

        assert_eq!(storage.name(first), "Photos");
        assert_eq!(storage.name(second), "");
        assert_eq!(storage.name(third), "Überweisung.pdf");
    }

    #[test]
    fn reorders_names_and_keeps_extensions() {
        let mut storage = NameStorage::with_capacity(4);
        let first = storage.add_name("a.jpg");
        let second = storage.add_name("Überweisung.pdf");
        let third = storage.add_name("");
        let jpg = storage.intern_extension_of("a.jpg");

        let reordered = storage.into_reordered([third, second, first, second].into_iter());

        assert_eq!(reordered.name_count(), 4);
        assert_eq!(reordered.name_at(0), "");
        assert_eq!(reordered.name_at(1), "Überweisung.pdf");
        assert_eq!(reordered.name_at(2), "a.jpg");
        assert_eq!(reordered.name_at(3), "Überweisung.pdf");
        assert_eq!(reordered.name_at(4), "");
        assert_eq!(reordered.interned_extension_id("jpg"), Some(jpg));
        assert_eq!(reordered.extension(jpg), Some("jpg"));
    }

    #[test]
    fn interns_extensions_case_insensitively() {
        let mut storage = NameStorage::with_capacity(0);

        let lower = storage.intern_extension_of("holiday.jpg");
        let upper = storage.intern_extension_of("IMG_0001.JPG");
        let other = storage.intern_extension_of("notes.txt");

        assert_eq!(lower, upper);
        assert_ne!(lower, other);
        assert_eq!(storage.extension(lower), Some("jpg"));
        assert_eq!(
            storage.extension_category(lower),
            Some(FileCategory::Images)
        );
        assert_eq!(
            storage.extension_category(other),
            Some(FileCategory::Documents)
        );
    }

    #[test]
    fn recognizes_names_without_an_extension() {
        let mut storage = NameStorage::with_capacity(0);

        assert_eq!(storage.intern_extension_of("Makefile"), ExtensionId::NONE);
        assert_eq!(storage.intern_extension_of("trailing."), ExtensionId::NONE);
        assert_eq!(
            storage.intern_extension_of("archive.this is not an extension"),
            ExtensionId::NONE
        );
        assert_eq!(storage.extension(ExtensionId::NONE), None);
        let dotfile = storage.intern_extension_of(".gitignore");
        assert_eq!(storage.extension(dotfile), Some("gitignore"));
    }
}
