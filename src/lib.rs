// Copyright 2016 Dtoa Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

mod diyf64;
mod f64toa;

use std::io;

#[inline]
pub fn write<W: io::Write, V: Floating>(wr: &mut W, value: V) -> io::Result<()> {
    value.write(wr)
}

pub trait Floating {
    fn write<W: io::Write>(self, &mut W) -> io::Result<()>;
}

impl Floating for f64 {
    fn write<W: io::Write>(self, wr: &mut W) -> io::Result<()> {
        unsafe { f64toa::dtoa(wr, self) }
    }
}

impl Floating for f32 {
    fn write<W: io::Write>(self, wr: &mut W) -> io::Result<()> {
        unsafe { f64toa::dtoa(wr, self as f64) }
    }
}
