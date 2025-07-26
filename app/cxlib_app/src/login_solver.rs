use crate::{GetKeyStr, global_multimap::GlobalMultimapItem};
use cxlib_internal::types::{LoginSolverTrait, UntypedLoginSolver};

pub type LoginSolverGetter<'gm, 'str, UserProtocol> =
    GlobalMultimapItem<'gm, 'str, UntypedLoginSolver<UserProtocol>>;

impl<T: LoginSolverTrait> GetKeyStr for T {
    #[inline]
    fn key_str(&self) -> &str {
        self.login_type()
    }
}
