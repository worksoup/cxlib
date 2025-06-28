use crate::global_multimap::GlobalMultimapItem;
use cxlib_internal::types::UntypedLoginSolver;

pub type LoginSolverGetter<'gm, 'str, UserProtocol> =
    GlobalMultimapItem<'gm, 'str, UntypedLoginSolver<UserProtocol>>;
