mod samsungtv;
mod specialformp3;

pub use samsungtv::{SamsungTv, SamsungTvButton};
pub use specialformp3::{SpecialForMp3, SpecialForMp3Button};

#[macro_export]
macro_rules! nec_buttons {
    ($buttonenum:tt, [$( ($cmd:expr, $name:tt) ),* $(,)?] ) => {

        #[allow(non_camel_case_types)]
        #[derive(Debug, Clone)]
        pub enum $buttonenum {
            $($name,)+
        }

        fn to_button(val: u8) -> Option<$buttonenum> {
            use $buttonenum::*;
            match val {
                $($cmd => Some($name),)+
                _ => None,
            }
        }

        fn from_button(button: $buttonenum) -> u8 {
            use $buttonenum::*;
            match button {
                $($name => $cmd,)+
            }
        }
    };
}

