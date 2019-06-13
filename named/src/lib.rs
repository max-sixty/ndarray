use ndarray::impl_linalg::Dot;
use ndarray::*;

// TODO: ideally this would be over an ArrayBase, rather than S, I think
pub struct NamedArray<S, D>
where
    S: RawData,
{
    array: ArrayBase<S, D>,
    dimensions: Vec<String>,
}

// impl NamedArray {

// }
// impl Dot for NamedArray {

// fn dot(&self, rhs: &NamedArray) ->
// }
