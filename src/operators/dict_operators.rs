use crate::{
    interpreter::InterpreterState,
    object::{Access, PostScriptDictionary},
    Object,
};

pub fn dict(state: &mut InterpreterState) -> crate::Result<()> {
    let capacity = state.pop_usize()?;

    let idx = state
        .dicts
        .insert(PostScriptDictionary::new(capacity, Access::default()));

    state.push(Object::Dictionary(idx));

    Ok(())
}

pub fn enddict(state: &mut InterpreterState) -> crate::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::error;

    #[test]
    fn test_dict() -> Result<(), Box<dyn error::Error>> {
        Ok(())
    }

    #[test]
    fn test_enddict() -> Result<(), Box<dyn error::Error>> {
        Ok(())
    }

    #[test]
    fn test_maxlength() -> Result<(), Box<dyn error::Error>> {
        Ok(())
    }

    #[test]
    fn test_begin() -> Result<(), Box<dyn error::Error>> {
        Ok(())
    }

    #[test]
    fn test_end() -> Result<(), Box<dyn error::Error>> {
        Ok(())
    }

    #[test]
    fn test_def() -> Result<(), Box<dyn error::Error>> {
        Ok(())
    }
}
