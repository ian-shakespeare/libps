use std::collections::HashMap;

use crate::{Mode, Object};

#[derive(Default)]
pub struct DictionaryObject {
    inner: HashMap<Object, Object>,
    pub(crate) mode: Mode,
}
