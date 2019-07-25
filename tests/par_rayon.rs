#![cfg(feature = "rayon")]

use ndarray::parallel::prelude::*;
use ndarray::prelude::*;

const M: usize = 1024 * 10;
const N: usize = 100;

#[test]
fn test_axis_iter() {
    let mut a = Array2::<f64>::zeros((M, N));
    for (i, mut v) in a.axis_iter_mut(Axis(0)).enumerate() {
        v.fill(i as _);
    }
    assert_eq!(a.axis_iter(Axis(0)).len(), M);
    let s: f64 = a.axis_iter(Axis(0)).into_par_iter().map(|x| x.sum()).sum();
    println!("{:?}", a.slice(s![..10, ..5]));
    assert_eq!(s, a.sum());
}

#[test]
#[cfg(feature = "approx")]
fn test_axis_iter_mut() {
    use approx::assert_abs_diff_eq;
    let mut a = Array::linspace(0., 1.0f64, M * N)
        .into_shape((M, N))
        .unwrap();
    let b = a.mapv(|x| x.exp());
    a.axis_iter_mut(Axis(0))
        .into_par_iter()
        .for_each(|mut v| v.mapv_inplace(|x| x.exp()));
    println!("{:?}", a.slice(s![..10, ..5]));
    assert_abs_diff_eq!(a, b, epsilon = 0.001);
}

#[test]
fn test_regular_iter() {
    let mut a = Array2::<f64>::zeros((M, N));
    for (i, mut v) in a.axis_iter_mut(Axis(0)).enumerate() {
        v.fill(i as _);
    }
    let s: f64 = a.view().into_par_iter().map(|&x| x).sum();
    println!("{:?}", a.slice(s![..10, ..5]));
    assert_eq!(s, a.sum());
}

#[test]
fn test_regular_iter_collect() {
    let mut a = Array2::<f64>::zeros((M, N));
    for (i, mut v) in a.axis_iter_mut(Axis(0)).enumerate() {
        v.fill(i as _);
    }
    let v = a.view().into_par_iter().map(|&x| x).collect::<Vec<_>>();
    assert_eq!(v.len(), a.len());
}
