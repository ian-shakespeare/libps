use std::{
    io::{StdinLock, StdoutLock},
    marker::PhantomData,
};

use crate::{
    dictionary::DictionaryObject,
    memory::VirtualMemory,
    object::{Access, Composite},
    Error, ErrorKind, Mode, Object,
};

enum MemoryRegion {
    Global,
    Local,
}

pub struct Context {
    pub operand_stack: Vec<Object>,
    pub dictionary_stack: Vec<DictionaryObject>,
    pub execution_stack: Vec<Object>,
    pub active_memory: MemoryRegion,
    pub global_memory: VirtualMemory,
    pub local_memory: VirtualMemory,
    pub stdin_file: StdinLock<'static>,
    pub stdout_file: StdoutLock<'static>,
    pub graphics_state: u8, // todo
}

impl Context {
    pub(crate) fn create_composite<T: Into<Vec<u8>>>(
        &mut self,
        value: T,
        access: Access,
        mode: Mode,
    ) -> Composite<T> {
        let address = self.memory_mut().allocate(value);

        Composite {
            address,
            access,
            mode,
            _marker: PhantomData,
        }
    }

    pub(crate) fn get_composite_value<T: TryFrom<Vec<u8>>>(
        &self,
        composite: Composite<T>,
    ) -> crate::Result<T> {
        let data = self.memory().deref(composite.address());

        T::try_from(data).map_err(|_| {
            Error::new(
                ErrorKind::VmError,
                "failed to retrieve composite from memory. found bytes are invalid.",
            )
        })
    }

    fn memory(&self) -> &VirtualMemory {
        match self.active_memory {
            MemoryRegion::Global => &self.global_memory,
            MemoryRegion::Local => &self.local_memory,
        }
    }

    fn memory_mut(&mut self) -> &mut VirtualMemory {
        match self.active_memory {
            MemoryRegion::Global => &mut self.global_memory,
            MemoryRegion::Local => &mut self.local_memory,
        }
    }
}
