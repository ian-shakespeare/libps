#[derive(Clone, Copy, Debug, Default)]
pub enum Access {
    #[default]
    Unlimited,
    ReadOnly,
    ExecuteOnly,
    None,
}

impl Access {
    pub fn is_read_only(&self) -> bool {
        matches!(self, Access::ReadOnly)
    }

    pub fn is_exec_only(&self) -> bool {
        matches!(self, Access::ExecuteOnly)
    }

    pub fn has_no_access(&self) -> bool {
        matches!(self, Access::None)
    }

    pub fn is_writeable(&self) -> bool {
        matches!(self, Access::Unlimited)
    }

    pub fn is_readable(&self) -> bool {
        self.is_writeable() || self.is_read_only()
    }
}
