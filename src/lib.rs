// Copyright 2016 Dtoa Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use] mod diyfp;
#[macro_use] mod dtoa;

use std::{io, mem, ops, ptr, slice};

#[inline]
pub fn write<W: io::Write, V: Floating>(wr: &mut W, value: V) -> io::Result<()> {
    value.write(wr)
}

pub trait Floating {
    fn write<W: io::Write>(self, &mut W) -> io::Result<()>;
}

impl Floating for f32 {
    fn write<W: io::Write>(self, wr: &mut W) -> io::Result<()> {
        dtoa! {
            floating_type: f32,
            significand_type: u32,
            exponent_type: i32,

            diy_significand_size: 32,
            significand_size: 23,
            exponent_bias: 0x7F,
            mask_type: u32,
            exponent_mask: 0x7F800000,
            significand_mask: 0x007FFFFF,
            hidden_bit: 0x00800000,
            cached_powers_f: CACHED_POWERS_F_32,
            cached_powers_e: CACHED_POWERS_E_32,
        };
        unsafe { dtoa(wr, self) }
    }
}

impl Floating for f64 {
    fn write<W: io::Write>(self, wr: &mut W) -> io::Result<()> {
        dtoa! {
            floating_type: f64,
            significand_type: u64,
            exponent_type: isize,

            diy_significand_size: 64,
            significand_size: 52,
            exponent_bias: 0x3FF,
            mask_type: u64,
            exponent_mask: 0x7FF0000000000000,
            significand_mask: 0x000FFFFFFFFFFFFF,
            hidden_bit: 0x0010000000000000,
            cached_powers_f: CACHED_POWERS_F_64,
            cached_powers_e: CACHED_POWERS_E_64,
        };
        unsafe { dtoa(wr, self) }
    }
}

////////////////////////////////////////////////////////////////////////////////

const MAX_DECIMAL_PLACES: isize = 324;

static DEC_DIGITS_LUT: &'static [u8] =
    b"0001020304050607080910111213141516171819\
      2021222324252627282930313233343536373839\
      4041424344454647484950515253545556575859\
      6061626364656667686970717273747576777879\
      8081828384858687888990919293949596979899";

// 10^-348, 10^-340, ..., 10^340
static CACHED_POWERS_F_32: [u32; 87] = [
    0xfa8fd5a0, 0xbaaee180, 0x8b16fb20, 0xcf42894a,
    0x9a6bb0aa, 0xe61acf03, 0xab70fe18, 0xff77b1fd,
    0xbe5691ef, 0x8dd01fae, 0xd3515c28, 0x9d71ac90,
    0xea9c2277, 0xaecc4991, 0x823c1279, 0xc2109436,
    0x9096ea6f, 0xd77485cb, 0xa086cfce, 0xef340a98,
    0xb23867fb, 0x84c8d4e0, 0xc5dd4427, 0x936b9fcf,
    0xdbac6c24, 0xa3ab6658, 0xf3e2f894, 0xb5b5ada9,
    0x87625f05, 0xc9bcff60, 0x964e858d, 0xdff97724,
    0xa6dfbda0, 0xf8a95fd0, 0xb9447094, 0x8a08f0f9,
    0xcdb02555, 0x993fe2c7, 0xe45c10c4, 0xaa242499,
    0xfd87b5f3, 0xbce50865, 0x8cbccc09, 0xd1b71759,
    0x9c400000, 0xe8d4a510, 0xad78ebc6, 0x813f3979,
    0xc097ce7c, 0x8f7e32ce, 0xd5d238a5, 0x9f4f2726,
    0xed63a232, 0xb0de6539, 0x83c7088e, 0xc45d1df9,
    0x924d692d, 0xda01ee64, 0xa26da39a, 0xf209787c,
    0xb454e4a1, 0x865b8692, 0xc83553c6, 0x952ab45d,
    0xde469fbe, 0xa59bc235, 0xf6c69a73, 0xb7dcbf53,
    0x88fcf318, 0xcc20ce9c, 0x98165af3, 0xe2a0b5dd,
    0xa8d9d153, 0xfb9b7cda, 0xbb764c4d, 0x8bab8ef0,
    0xd01fef11, 0x9b10a4e6, 0xe7109bfc, 0xac2820d9,
    0x80444b5e, 0xbf21e440, 0x8e679c2f, 0xd433179e,
    0x9e19db93, 0xeb96bf6f, 0xaf87023c,
];

static CACHED_POWERS_E_32: [i16; 87] = [
    -1188, -1161, -1134, -1108, -1081, -1055, -1028, -1002,  -975,  -948,
     -922,  -895,  -869,  -842,  -815,  -789,  -762,  -736,  -709,  -683,
     -656,  -629,  -603,  -576,  -550,  -523,  -497,  -470,  -443,  -417,
     -390,  -364,  -337,  -311,  -284,  -257,  -231,  -204,  -178,  -151,
     -125,   -98,   -71,   -45,   -18,     8,    35,    62,    88,   115,
      141,   168,   194,   221,   248,   274,   301,   327,   354,   380,
      407,   434,   460,   487,   513,   540,   566,   593,   620,   646,
      673,   699,   726,   752,   779,   806,   832,   859,   885,   912,
      939,   965,   992,  1018,  1045,  1071,  1098,
];

// 10^-348, 10^-340, ..., 10^340
static CACHED_POWERS_F_64: [u64; 87] = [
    0xfa8fd5a0081c0288, 0xbaaee17fa23ebf76,
    0x8b16fb203055ac76, 0xcf42894a5dce35ea,
    0x9a6bb0aa55653b2d, 0xe61acf033d1a45df,
    0xab70fe17c79ac6ca, 0xff77b1fcbebcdc4f,
    0xbe5691ef416bd60c, 0x8dd01fad907ffc3c,
    0xd3515c2831559a83, 0x9d71ac8fada6c9b5,
    0xea9c227723ee8bcb, 0xaecc49914078536d,
    0x823c12795db6ce57, 0xc21094364dfb5637,
    0x9096ea6f3848984f, 0xd77485cb25823ac7,
    0xa086cfcd97bf97f4, 0xef340a98172aace5,
    0xb23867fb2a35b28e, 0x84c8d4dfd2c63f3b,
    0xc5dd44271ad3cdba, 0x936b9fcebb25c996,
    0xdbac6c247d62a584, 0xa3ab66580d5fdaf6,
    0xf3e2f893dec3f126, 0xb5b5ada8aaff80b8,
    0x87625f056c7c4a8b, 0xc9bcff6034c13053,
    0x964e858c91ba2655, 0xdff9772470297ebd,
    0xa6dfbd9fb8e5b88f, 0xf8a95fcf88747d94,
    0xb94470938fa89bcf, 0x8a08f0f8bf0f156b,
    0xcdb02555653131b6, 0x993fe2c6d07b7fac,
    0xe45c10c42a2b3b06, 0xaa242499697392d3,
    0xfd87b5f28300ca0e, 0xbce5086492111aeb,
    0x8cbccc096f5088cc, 0xd1b71758e219652c,
    0x9c40000000000000, 0xe8d4a51000000000,
    0xad78ebc5ac620000, 0x813f3978f8940984,
    0xc097ce7bc90715b3, 0x8f7e32ce7bea5c70,
    0xd5d238a4abe98068, 0x9f4f2726179a2245,
    0xed63a231d4c4fb27, 0xb0de65388cc8ada8,
    0x83c7088e1aab65db, 0xc45d1df942711d9a,
    0x924d692ca61be758, 0xda01ee641a708dea,
    0xa26da3999aef774a, 0xf209787bb47d6b85,
    0xb454e4a179dd1877, 0x865b86925b9bc5c2,
    0xc83553c5c8965d3d, 0x952ab45cfa97a0b3,
    0xde469fbd99a05fe3, 0xa59bc234db398c25,
    0xf6c69a72a3989f5c, 0xb7dcbf5354e9bece,
    0x88fcf317f22241e2, 0xcc20ce9bd35c78a5,
    0x98165af37b2153df, 0xe2a0b5dc971f303a,
    0xa8d9d1535ce3b396, 0xfb9b7cd9a4a7443c,
    0xbb764c4ca7a44410, 0x8bab8eefb6409c1a,
    0xd01fef10a657842c, 0x9b10a4e5e9913129,
    0xe7109bfba19c0c9d, 0xac2820d9623bf429,
    0x80444b5e7aa7cf85, 0xbf21e44003acdd2d,
    0x8e679c2f5e44ff8f, 0xd433179d9c8cb841,
    0x9e19db92b4e31ba9, 0xeb96bf6ebadf77d9,
    0xaf87023b9bf0ee6b,
];
static CACHED_POWERS_E_64: [i16; 87] = [
    -1220, -1193, -1166, -1140, -1113, -1087, -1060, -1034, -1007,  -980,
    -954,   -927,  -901,  -874,  -847,  -821,  -794,  -768,  -741,  -715,
    -688,   -661,  -635,  -608,  -582,  -555,  -529,  -502,  -475,  -449,
    -422,   -396,  -369,  -343,  -316,  -289,  -263,  -236,  -210,  -183,
    -157,   -130,  -103,   -77,   -50,   -24,     3,    30,    56,    83,
     109,    136,   162,   189,   216,   242,   269,   295,   322,   348,
     375,    402,   428,   455,   481,   508,   534,   561,   588,   614,
     641,    667,   694,   720,   747,   774,   800,   827,   853,   880,
     907,    933,   960,   986,  1013,  1039,  1066,
];
