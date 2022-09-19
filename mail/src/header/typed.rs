// SPDX-License-Identifier: GPL-3.0-or-later

mod from;
mod sender;
mod subject;
mod to;

pub use from::*;
pub use sender::*;
pub use subject::*;
pub use to::*;

use super::HeaderName;

pub trait TypedHeader: Sized {
    type Error;
    const NAME: HeaderName<'static>;

    fn decode(encoded: &str) -> Result<Self, Self::Error>;
}
