use crate::{
    container::Container,
    object::{Composite, DictionaryObject, Object},
    rand::RandomNumberGenerator,
    Error, ErrorKind,
};

#[derive(Default)]
pub struct Context {
    rng: RandomNumberGenerator,
    pub operand_stack: Vec<Object>,
    local_mem: Container<Composite>,
    global_mem: Container<Composite>,
    dict_stack: Vec<usize>,
}

impl<'a> Context {
    pub fn mem(&self) -> &Container<Composite> {
        // TODO: Switch between local and global as needed
        &self.local_mem
    }

    pub fn mem_mut(&mut self) -> &mut Container<Composite> {
        // TODO: Switch between local and global as needed
        &mut self.local_mem
    }

    pub fn find_def<S>(&'a self, key: S) -> crate::Result<&'a Object>
    where
        S: Into<&'a str>,
    {
        let key: &str = key.into();

        for idx in self.dict_stack.iter().rev() {
            if let Some(comp) = self.mem().get(*idx) {
                let dict: &DictionaryObject = comp.try_into()?;

                if let Some(obj) = dict.get(key) {
                    return Ok(obj);
                }
            }
        }

        Err(Error::new(ErrorKind::Undefined, key))
    }
}
