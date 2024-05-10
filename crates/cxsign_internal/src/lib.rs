use cxsign_activity::sign;
pub use cxsign_activity::{Activity, OtherActivity};
pub use cxsign_error::*;
use cxsign_signner as signner;
pub use cxsign_store::UnameAndEncPwdPair;
pub use cxsign_types::{
    Course, Location, LocationAndAliasesPair, LocationPreprocessorTrait, LocationWithRange, Photo,
};
pub use cxsign_user::{Session, UserCookies};
pub use sign::*;
pub use signner::{
    DefaultGestureOrSigncodeSignner, DefaultLocationSignner, DefaultNormalOrRawSignner,
    DefaultPhotoSignner, DefaultQrCodeSignner, SignnerTrait,
};

pub mod protocol {
    pub use cxsign_activity::protocol::*;
    pub use cxsign_captcha::protocol::*;
    pub use cxsign_login::protocol::*;
    pub use cxsign_pan::protocol::*;
    pub use cxsign_types::protocol::*;
    pub use cxsign_user::protocol::*;
}

pub mod store {
    pub use cxsign_store::{DataBase, DataBaseTableTrait};
    pub mod tables {
        pub use cxsign_store::{AccountTable, AliasTable, ExcludeTable};
        pub use cxsign_types::{CourseTable, LocationTable};
    }
}
pub mod utils {
    pub use crate::signner::utils::*;
    pub use cxsign_captcha::utils::*;
    pub use cxsign_dir::*;
    pub use cxsign_imageproc::*;
    pub use cxsign_login::{des_enc, load_json, login_enc};
    pub use cxsign_types::{
        do_location_preprocessor, set_boxed_location_preprocessor, set_location_preprocessor,
    };
    pub use cxsign_utils::*;
}
