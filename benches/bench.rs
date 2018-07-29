#![feature(test)]

extern crate dtoa;
extern crate test;

macro_rules! benches {
    ($($name:ident($value:expr),)*) => {
        mod bench_dtoa {
            use test::{Bencher, black_box};
            $(
                #[bench]
                fn $name(b: &mut Bencher) {
                    use dtoa;

                    let mut buf = Vec::with_capacity(20);

                    b.iter(|| {
                        buf.clear();
                        dtoa::write(&mut buf, black_box($value)).unwrap()
                    });
                }
            )*
        }

        /*
        mod bench_fmt {
            use test::{Bencher, black_box};
            $(
                #[bench]
                fn $name(b: &mut Bencher) {
                    use std::io::Write;

                    let mut buf = Vec::with_capacity(20);

                    b.iter(|| {
                        buf.clear();
                        write!(&mut buf, "{}", black_box($value)).unwrap()
                    });
                }
            )*
        }
        */

        mod bench_dtolnay_ryu {
            extern crate dtolnay_ryu;
            use test::{Bencher, black_box};

            $(
                #[bench]
                fn $name(b: &mut Bencher) {
                    use std::io::Write;

                    let mut buf = Vec::new();

                    b.iter(|| {
                        buf.clear();
                        let mut buffer = dtolnay_ryu::Buffer::new();
                        let s = buffer.format(black_box($value));
                        buf.write_all(s.as_bytes()).unwrap();
                    });
                }
            )*
        }

        mod bench_c_ryu {
            extern crate ryu;
            use test::{Bencher, black_box};

            trait Ryu: Sized {
                type Buffer;
                fn w(self) -> Self::Buffer;
            }

            impl Ryu for f32 {
                type Buffer = ryu::F32String;
                fn w(self) -> Self::Buffer {
                    ryu::f2s(self)
                }
            }

            impl Ryu for f64 {
                type Buffer = ryu::F64String;
                fn w(self) -> Self::Buffer {
                    ryu::d2s(self)
                }
            }

            $(
                #[bench]
                fn $name(b: &mut Bencher) {
                    use std::io::Write;

                    let mut buf = Vec::new();

                    b.iter(|| {
                        buf.clear();
                        let s = Ryu::w(black_box($value));
                        buf.write_all(s.as_bytes()).unwrap();
                    });
                }
            )*
        }

        mod bench_float_fast_print {
            extern crate float_fast_print;
            use test::{Bencher, black_box};

            trait FloatFastPrint {
                fn w(self, buf: &mut Vec<u8>);
            }

            impl FloatFastPrint for f32 {
                fn w(self, buf: &mut Vec<u8>) {
                    float_fast_print::write_f32_shortest(buf, self).unwrap();
                }
            }

            impl FloatFastPrint for f64 {
                fn w(self, buf: &mut Vec<u8>) {
                    float_fast_print::write_f64_shortest(buf, self).unwrap();
                }
            }

            $(
                #[bench]
                fn $name(b: &mut Bencher) {
                    let mut buf = Vec::new();

                    b.iter(|| {
                        buf.clear();
                        FloatFastPrint::w(black_box($value), &mut buf);
                    });
                }
            )*
        }
    }
}

benches!(
    bench_0_f64(0f64),
    bench_short_f64(0.1234f64),
    bench_e_f64(2.718281828459045f64),
    bench_max_f64(::std::f64::MAX),

    bench_0_f32(0f32),
    bench_short_f32(0.1234f32),
    bench_e_f32(2.718281828459045f32),
    bench_max_f32(::std::f32::MAX),
);
