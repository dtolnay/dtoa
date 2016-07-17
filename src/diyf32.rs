// Copyright 2016 Dtoa Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::{mem, ops};

const DIY_SIGNIFICAND_SIZE: isize = 32;
const SP_SIGNIFICAND_SIZE: isize = 23;
const SP_EXPONENT_BIAS: isize = 0x7F + SP_SIGNIFICAND_SIZE;
const SP_MIN_EXPONENT: isize = -SP_EXPONENT_BIAS;
const SP_EXPONENT_MASK: u32 = 0x7F800000;
const SP_SIGNIFICAND_MASK: u32 = 0x007FFFFF;
const SP_HIDDEN_BIT: u32 = 0x00800000;

#[derive(Copy, Clone, Debug)]
pub struct DiyFp {
    pub f: u32,
    pub e: isize,
}

impl DiyFp {
    pub fn new(f: u32, e: isize) -> Self {
        DiyFp { f: f, e: e }
    }

    /*
    explicit DiyFp(double d) {
        union {
            double d;
            uint64_t u64;
        } u = { d };

        int biased_e = static_cast<int>((u.u64 & kDpExponentMask) >> kDpSignificandSize);
        uint64_t significand = (u.u64 & kDpSignificandMask);
        if (biased_e != 0) {
            f = significand + kDpHiddenBit;
            e = biased_e - kDpExponentBias;
        }
        else {
            f = significand;
            e = kDpMinExponent + 1;
        }
    }
    */
    pub unsafe fn from_f32(d: f32) -> Self {
        let u: u32 = mem::transmute(d);

        let biased_e = ((u & SP_EXPONENT_MASK) >> SP_SIGNIFICAND_SIZE) as isize;
        let significand = u & SP_SIGNIFICAND_MASK;
        if biased_e != 0 {
            DiyFp {
                f: significand + SP_HIDDEN_BIT,
                e: biased_e - SP_EXPONENT_BIAS,
            }
        } else {
            DiyFp {
                f: significand,
                e: SP_MIN_EXPONENT + 1,
            }
        }
    }

    /*
    DiyFp Normalize() const {
        DiyFp res = *this;
        while (!(res.f & (static_cast<uint64_t>(1) << 63))) {
            res.f <<= 1;
            res.e--;
        }
        return res;
    }
    */
    pub fn normalize(self) -> DiyFp {
        let mut res = self;
        while (res.f & (1u32 << 31)) == 0 {
            res.f <<= 1;
            res.e -= 1;
        }
        res
    }

    /*
    DiyFp NormalizeBoundary() const {
        DiyFp res = *this;
        while (!(res.f & (kDpHiddenBit << 1))) {
            res.f <<= 1;
            res.e--;
        }
        res.f <<= (kDiySignificandSize - kDpSignificandSize - 2);
        res.e = res.e - (kDiySignificandSize - kDpSignificandSize - 2);
        return res;
    }
    */
    fn normalize_boundary(self) -> DiyFp {
        let mut res = self;
        while (res.f & SP_HIDDEN_BIT << 1) == 0 {
            res.f <<= 1;
            res.e -= 1;
        }
        res.f <<= DIY_SIGNIFICAND_SIZE - SP_SIGNIFICAND_SIZE - 2;
        res.e -= DIY_SIGNIFICAND_SIZE - SP_SIGNIFICAND_SIZE - 2;
        res
    }

    /*
    void NormalizedBoundaries(DiyFp* minus, DiyFp* plus) const {
        DiyFp pl = DiyFp((f << 1) + 1, e - 1).NormalizeBoundary();
        DiyFp mi = (f == kDpHiddenBit) ? DiyFp((f << 2) - 1, e - 2) : DiyFp((f << 1) - 1, e - 1);
        mi.f <<= mi.e - pl.e;
        mi.e = pl.e;
        *plus = pl;
        *minus = mi;
    }
    */
    pub fn normalized_boundaries(self) -> (DiyFp, DiyFp) {
        let pl = DiyFp::new((self.f << 1) + 1, self.e - 1).normalize_boundary();
        let mut mi = if self.f == SP_HIDDEN_BIT {
            DiyFp::new((self.f << 2) - 1, self.e - 2)
        } else {
            DiyFp::new((self.f << 1) - 1, self.e - 1)
        };
        mi.f <<= mi.e - pl.e;
        mi.e = pl.e;
        (mi, pl)
    }
}

impl ops::Sub for DiyFp {
    type Output = DiyFp;
    fn sub(self, rhs: DiyFp) -> DiyFp {
        DiyFp {
            f: self.f - rhs.f,
            e: self.e,
        }
    }
}

impl ops::Mul for DiyFp {
    type Output = DiyFp;
    fn mul(self, rhs: DiyFp) -> DiyFp {
        let mut tmp = self.f as u64 * rhs.f as u64;
        tmp += 1u64 << 31; // mult_round
        DiyFp {
            f: (tmp >> 32) as u32,
            e: self.e + rhs.e + 32,
        }
    }
}

fn get_cached_power_by_index(index: usize) -> DiyFp {
    // 10^-348, 10^-340, ..., 10^340
    static CACHED_POWERS_F: [u32; 87] = [
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
    static CACHED_POWERS_E: [i16; 87] = [
        -1188, -1161, -1134, -1108, -1081, -1055, -1028, -1002, -975, -948,
         -922,  -895,  -869,  -842,  -815,  -789,  -762,  -736, -709, -683,
         -656,  -629,  -603,  -576,  -550,  -523,  -497,  -470, -443, -417,
         -390,  -364,  -337,  -311,  -284,  -257,  -231,  -204, -178, -151,
         -125,   -98,   -71,   -45,   -18,     8,    35,    62,   88,  115,
          141,   168,   194,   221,   248,   274,   301,   327,  354,  380,
          407,   434,   460,   487,   513,   540,   566,   593,  620,  646,
          673,   699,   726,   752,   779,   806,   832,   859,  885,  912,
          939,   965,   992,  1018,  1045,  1071,  1098,
    ];
    DiyFp::new(CACHED_POWERS_F[index], CACHED_POWERS_E[index] as isize)
}

/*
inline DiyFp GetCachedPower(int e, int* K) {
    //int k = static_cast<int>(ceil((-61 - e) * 0.30102999566398114)) + 374;
    double dk = (-61 - e) * 0.30102999566398114 + 347;  // dk must be positive, so can do ceiling in positive
    int k = static_cast<int>(dk);
    if (dk - k > 0.0)
        k++;

    unsigned index = static_cast<unsigned>((k >> 3) + 1);
    *K = -(-348 + static_cast<int>(index << 3));    // decimal exponent no need lookup table

    return GetCachedPowerByIndex(index);
}
*/
#[inline]
pub fn get_cached_power(e: isize) -> (DiyFp, isize) {
    let dk = (-29 - e) as f64 * 0.30102999566398114f64 + 347f64; // dk must be positive, so can do ceiling in positive
    let mut k = dk as isize;
    if dk - k as f64 > 0.0 {
        k += 1;
    }

    let index = ((k >> 3) + 1) as usize;
    let k = -(-348 + (index << 3) as isize); // decimal exponent no need lookup table

    (get_cached_power_by_index(index), k)
}
