mod samsungtv;
mod specialformp3;

pub use samsungtv::{SamsungTv};
pub use specialformp3::{SpecialForMp3};

#[macro_export]
macro_rules! standard_mapping {
    ( [$( ($cmd:expr, $name:tt) ),* $(,)?] ) => {

        fn to_button(val: u8) -> Option<StandardButton> {
            match val {
                $($cmd => Some(StandardButton::$name),)+
                _ => None,
            }
        }

        fn from_button(button: StandardButton) -> Option<u8> {
            match button {
                $(StandardButton::$name => Some($cmd),)+
                _ => None,
            }
        }
    };
}

