#![feature(test)]
#![allow(
    clippy::approx_constant,
    clippy::excessive_precision,
    clippy::unreadable_literal
)]

extern crate test;

macro_rules! benches {
    ($($name:ident($value:expr),)*) => {
        mod bench_dtoa {
            use std::hint;
            use test::Bencher;

            $(
                #[bench]
                fn $name(b: &mut Bencher) {
                    let mut buffer = dtoa::Buffer::new();

                    b.iter(|| {
                        let printed = buffer.format_finite(hint::black_box($value));
                        hint::black_box(printed);
                    });
                }
            )*
        }

        mod bench_fmt {
            use std::hint;
            use std::io::Write;
            use test::Bencher;

            $(
                #[bench]
                fn $name(b: &mut Bencher) {
                    let mut buf = Vec::with_capacity(20);

                    b.iter(|| {
                        buf.clear();
                        write!(&mut buf, "{}", hint::black_box($value)).unwrap();
                        hint::black_box(&buf);
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
    bench_max_f64(f64::MAX),
    bench_0_f32(0f32),
    bench_short_f32(0.1234f32),
    bench_e_f32(2.718281828459045f32),
    bench_max_f32(f32::MAX),
);
