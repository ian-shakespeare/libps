use std::{fmt, slice};

use crate::object::{Access, Mode, Object};

#[derive(Debug)]
pub struct ArrayObject {
    access: Access,
    inner: Vec<Object>,
    pub(crate) mode: Mode,
}

impl ArrayObject {
    pub fn new(objs: Vec<Object>, access: Access, mode: Mode) -> Self {
        Self {
            inner: objs,
            access,
            mode,
        }
    }

    pub fn access(&self) -> Access {
        self.access
    }

    pub fn iter(&self) -> slice::Iter<'_, Object> {
        self.inner.iter()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

impl fmt::Display for ArrayObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (l_delim, r_delim) = match self.mode {
            Mode::Executable => ("{ ", "}"),
            Mode::Literal => ("[ ", "]"),
        };

        l_delim.fmt(f)?;
        for obj in self.iter() {
            obj.fmt(f)?;
            " ".fmt(f)?;
        }
        r_delim.fmt(f)?;

        Ok(())
    }
}
