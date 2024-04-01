
use crate::datatype::BuilderDataType;
use crate::errors::BuilderError;

pub struct Closure<'de> {
    pub(crate) args: Vec<BuilderDataType<'de>>,
    pub(crate) index: usize,
}


impl<'de> Closure<'de> {
    pub(crate) fn get_argument(&self, a: usize) -> Result<&BuilderDataType<'de>, BuilderError> {
        if let Some(a) = self.args.get(a) {
            Ok(a)
        } else {
            Err(BuilderError::InvalidFunctionArgument)
        }
    }
    pub(crate) fn clone_argument(&self, a: usize) -> Result<BuilderDataType<'de>, BuilderError> {
        if let Some(a) = self.args.get(a) {
            Ok(a.clone())
        } else {
            Err(BuilderError::InvalidFunctionArgument)
        }
    }
    pub(crate) fn take_from_argument(&mut self, a: usize) -> Result<BuilderDataType<'de>, BuilderError> {
        if let Some(a) = self.args.get_mut(a) {
            Ok(a.take_one())
        } else {
            Err(BuilderError::InvalidFunctionArgument)
        }
    }
    pub(crate) fn resolve(&mut self, b: BuilderDataType<'de>) -> Result<BuilderDataType<'de>, BuilderError> {
        match b {
            BuilderDataType::Argument(a) => self.clone_argument(a),
            BuilderDataType::TakeFromArgument(a) => self.take_from_argument(a),
            BuilderDataType::IfThenElse(v) => self.if_then_else(v),
            b => Ok(b),
        }
    }
    pub(crate) fn resolve_clone(&mut self, b: &BuilderDataType<'de>) -> Result<BuilderDataType<'de>, BuilderError> {
        match b {
            BuilderDataType::Argument(a) => self.clone_argument(*a),
            BuilderDataType::TakeFromArgument(a) => self.take_from_argument(*a),
            BuilderDataType::IfThenElse(v) => self.if_then_else_ref(v).cloned(),
            b => Ok(b.clone()),
        }
    }

    pub(crate) fn resolve_to_bool(&mut self, b: &BuilderDataType<'de>) -> Result<bool, BuilderError> {
        Ok(match b {
            BuilderDataType::Argument(a) => self.get_argument(*a)?.check_true(),
            BuilderDataType::TakeFromArgument(a) => self.take_from_argument(*a)?.check_true(),
            BuilderDataType::IfThenElse(v) => self.if_then_else_ref(v)?.check_true(),
            b => b.check_true(),
        })
    }

    pub(crate) fn if_then_else_ref<'a>(
        &mut self,
        v: &'a Vec<BuilderDataType<'de>>,
    ) -> Result<&'a BuilderDataType<'de>, BuilderError> {
        let mut i = v.iter();
        let Some(condition) = i.next() else {
            return Err(BuilderError::InvalidFunctionArgument);
        };
        let Some(if_true) = i.next() else {
            return Err(BuilderError::InvalidFunctionArgument);
        };
        let Some(if_false) = i.next() else {
            return Err(BuilderError::InvalidFunctionArgument);
        };
        if self.resolve_to_bool(condition)? {
            Ok(if_true)
        } else {
            Ok(if_false)
        }
    }
    pub(crate) fn if_then_else(
        &mut self,
        v: Vec<BuilderDataType<'de>>,
    ) -> Result<BuilderDataType<'de>, BuilderError> {
        let mut i = v.into_iter();
        let Some(condition) = i.next() else {
            return Err(BuilderError::InvalidFunctionArgument);
        };
        let Some(if_true) = i.next() else {
            return Err(BuilderError::InvalidFunctionArgument);
        };
        let Some(if_false) = i.next() else {
            return Err(BuilderError::InvalidFunctionArgument);
        };
        if self.resolve(condition)?.check_true() {
            Ok(if_true)
        } else {
            Ok(if_false)
        }
    }
}
