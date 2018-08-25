extern crate criterion;
extern crate rand;

use criterion::Criterion;
use rand::{ChaChaRng, Rng, SeedableRng};

mod plain {
    #[inline(always)]
    fn clip8(v: i16) -> u8 {
        v.min(255).max(0) as u8
    }

    pub fn add_coeffs(dst: &mut [u8], mut idx: usize, stride: usize, coeffs: &[i16]) {
        for y in 0..4 {
            for x in 0..4 {
                dst[idx + x] = clip8((dst[idx + x] as i16) + coeffs[x + y * 4]);
            }
            idx += stride;
        }
    }
}

mod kostya {
    #[inline(always)]
    fn mclip8(v: i32) -> u8 {
        v.min(255).max(0) as u8
    }

    pub fn add_coeffs(dst: &mut [u8], idx: usize, stride: usize, coeffs: &[i16]) {
        let out = &mut dst[idx..][..stride * 3 + 4];
        let mut sidx: usize = 0;
        for el in out.chunks_mut(stride).take(4) {
            assert!(el.len() >= 4);
            el[0] = mclip8((el[0] as i32) + (coeffs[0 + sidx] as i32));
            el[1] = mclip8((el[1] as i32) + (coeffs[1 + sidx] as i32));
            el[2] = mclip8((el[2] as i32) + (coeffs[2 + sidx] as i32));
            el[3] = mclip8((el[3] as i32) + (coeffs[3 + sidx] as i32));
            sidx += 4;
        }
    }
}

mod lu {
    #[inline(always)]
    fn clip8(v: i32) -> u8 {
        v.min(255).max(0) as u8
    }

    pub fn add_coeffs(dst: &mut [u8], idx: usize, stride: usize, coeffs: &[i16]) {
        let out = &mut dst[idx..][..stride * 3 + 4];
        let coeffs = &coeffs[..16];
        for (el, cf) in out.chunks_mut(stride).take(4).zip(coeffs.chunks(4)) {
            assert!(el.len() >= 4);
            assert!(cf.len() >= 4);
            el[0] = clip8((el[0] as i32) + (cf[0] as i32));
            el[1] = clip8((el[1] as i32) + (cf[1] as i32));
            el[2] = clip8((el[2] as i32) + (cf[2] as i32));
            el[3] = clip8((el[3] as i32) + (cf[3] as i32));
        }
    }
}

type AddCoeffs=fn(dst: &mut [u8], idx: usize, stride: usize, coeffs: &[i16]);

fn make_buffers(_stride: usize, blocks: usize) -> (Vec<u8>, Vec<i16>) {
    let mut rng = ChaChaRng::from_seed([0; 32]);

    let dst = (0..blocks).map(|_| rng.gen()).collect();
    let coeffs = (0..16).map(|_| rng.gen_range(-511, 511)).collect();

    (dst, coeffs)
}

fn bench_add_coeff(name: &str, blocks: usize, stride: usize, add_coeffs: AddCoeffs, c: &mut Criterion) {
    let (mut dst, coeffs) = make_buffers(stride, blocks);

    c.bench_function(&format!("{}_{}_{}", name, blocks, stride), move |b| {
        b.iter(|| {
            for y in 0 .. blocks / 4 / stride {
                for x in 0 .. stride / 4 {
                    let idx = x * 4 + y * 4 * stride;
                    add_coeffs(&mut dst, idx, stride, &coeffs);
                }
            }
        });
    });
}

fn main() {
    let blocks = 512 * 384;
    let stride = 512;

    criterion::init_logging();

    let mut c = Criterion::default().configure_from_args();
    bench_add_coeff("plain", blocks, stride, plain::add_coeffs, &mut c);
    bench_add_coeff("kostya", blocks, stride, kostya::add_coeffs, &mut c);
    bench_add_coeff("lu", blocks, stride, lu::add_coeffs, &mut c);

    criterion::Criterion::default()
        .configure_from_args()
        .final_summary();
}
