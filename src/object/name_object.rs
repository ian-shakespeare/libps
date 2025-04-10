use super::Mode;

#[derive(Clone, Debug)]
pub struct NameObject {
    inner: String,
    pub mode: Mode,
}

impl NameObject {
    pub fn new<S>(value: S, mode: Mode) -> Self
    where
        S: Into<String>,
    {
        Self {
            inner: value.into(),
            mode,
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl AsRef<str> for NameObject {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl<'a> From<&'a NameObject> for &'a str {
    fn from(value: &'a NameObject) -> Self {
        &value.inner
    }
}

impl<'a> From<&'a str> for NameObject {
    fn from(value: &'a str) -> Self {
        Self {
            inner: value.to_string(),
            mode: Mode::Executable,
        }
    }
}

impl PartialEq<str> for NameObject {
    fn eq(&self, other: &str) -> bool {
        self.inner == other
    }
}

impl PartialEq<NameObject> for NameObject {
    fn eq(&self, other: &NameObject) -> bool {
        self.inner == other.inner
    }
}
