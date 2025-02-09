use std::collections::HashMap;

use crate::{
    interpreter::InterpreterState,
    object::{Access, PostScriptDictionary},
    Error, ErrorKind, Object,
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
    let mut dict = HashMap::new();

    loop {
        let value = state.pop()?;
        if value.is_mark() {
            break;
        }

        let key = state.pop()?;
        if key.is_mark() {
            return Err(Error::from(ErrorKind::RangeCheck));
        }

        let key = key.to_string(state)?;

        dict.insert(key, value);
    }

    let idx = state.dicts.insert(dict.into());
    state.push(Object::Dictionary(idx));

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
