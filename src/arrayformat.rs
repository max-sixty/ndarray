// Copyright 2014-2016 bluss and ndarray developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use super::{ArrayBase, Axis, Data, Dimension, Ix, NdProducer};
use crate::aliases::Ix1;
use std::fmt;

const PRINT_ELEMENTS_LIMIT: Ix = 3;

fn format_1d_array<A, S, F>(
    view: &ArrayBase<S, Ix1>,
    f: &mut fmt::Formatter<'_>,
    mut format: F,
    limit: Ix,
) -> fmt::Result
where
    F: FnMut(&A, &mut fmt::Formatter<'_>) -> fmt::Result,
    S: Data<Elem = A>,
{
    let to_be_printed = to_be_printed(view.len(), limit);

    let n_to_be_printed = to_be_printed.len();

    write!(f, "[")?;
    for (j, index) in to_be_printed.into_iter().enumerate() {
        match index {
            PrintableCell::ElementIndex(i) => {
                format(&view[i], f)?;
                if j != n_to_be_printed - 1 {
                    write!(f, ", ")?;
                }
            }
            PrintableCell::Ellipses => write!(f, "..., ")?,
        }
    }
    write!(f, "]")?;
    Ok(())
}

enum PrintableCell {
    ElementIndex(usize),
    Ellipses,
}

// Returns what indexes should be printed for a certain axis.
// If the axis is longer than 2 * limit, a `Ellipses` is inserted
// where indexes are being omitted.
fn to_be_printed(length: usize, limit: usize) -> Vec<PrintableCell> {
    if length <= 2 * limit {
        (0..length).map(PrintableCell::ElementIndex).collect()
    } else {
        let mut v: Vec<PrintableCell> = (0..limit).map(PrintableCell::ElementIndex).collect();
        v.push(PrintableCell::Ellipses);
        v.extend((length - limit..length).map(PrintableCell::ElementIndex));
        v
    }
}

fn format_array<A, S, D, F>(
    view: &ArrayBase<S, D>,
    f: &mut fmt::Formatter<'_>,
    mut format: F,
    limit: Ix,
) -> fmt::Result
where
    F: FnMut(&A, &mut fmt::Formatter<'_>) -> fmt::Result + Clone,
    D: Dimension,
    S: Data<Elem = A>,
{
    // If any of the axes has 0 length, we return the same empty array representation
    // e.g. [[]] for 2-d arrays
    if view.shape().iter().any(|&x| x == 0) {
        write!(f, "{}{}", "[".repeat(view.ndim()), "]".repeat(view.ndim()))?;
        return Ok(());
    }
    match view.shape() {
        // If it's 0 dimensional, we just print out the scalar
        [] => format(view.iter().next().unwrap(), f)?,
        // We delegate 1-dimensional arrays to a specialized function
        [_] => format_1d_array(
            &view.view().into_dimensionality::<Ix1>().unwrap(),
            f,
            format,
            limit,
        )?,
        // For n-dimensional arrays, we proceed recursively
        shape => {
            // Cast into a dynamically dimensioned view
            // This is required to be able to use `index_axis`
            let view = view.view().into_dyn();
            // We start by checking what indexes from the first axis should be printed
            // We put a `None` in the middle if we are omitting elements
            let to_be_printed = to_be_printed(shape[0], limit);

            let n_to_be_printed = to_be_printed.len();

            write!(f, "[")?;
            for (j, index) in to_be_printed.into_iter().enumerate() {
                match index {
                    PrintableCell::ElementIndex(i) => {
                        // Proceed recursively with the (n-1)-dimensional slice
                        format_array(&view.index_axis(Axis(0), i), f, format.clone(), limit)?;
                        // We need to add a separator after each slice,
                        // apart from the last one
                        if j != n_to_be_printed - 1 {
                            write!(f, ",\n ")?
                        }
                    }
                    PrintableCell::Ellipses => write!(f, "...,\n ")?,
                }
            }
            write!(f, "]")?;
        }
    }
    Ok(())
}

// NOTE: We can impl other fmt traits here
/// Format the array using `Display` and apply the formatting parameters used
/// to each element.
///
/// The array is shown in multiline style.
impl<'a, A: fmt::Display, S, D: Dimension> fmt::Display for ArrayBase<S, D>
where
    S: Data<Elem = A>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_array(self, f, <_>::fmt, PRINT_ELEMENTS_LIMIT)
    }
}

/// Format the array using `Debug` and apply the formatting parameters used
/// to each element.
///
/// The array is shown in multiline style.
impl<'a, A: fmt::Debug, S, D: Dimension> fmt::Debug for ArrayBase<S, D>
where
    S: Data<Elem = A>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Add extra information for Debug
        format_array(self, f, <_>::fmt, PRINT_ELEMENTS_LIMIT)?;
        write!(
            f,
            " shape={:?}, strides={:?}, layout={:?}",
            self.shape(),
            self.strides(),
            layout = self.view().layout()
        )?;
        match D::NDIM {
            Some(ndim) => write!(f, ", const ndim={}", ndim)?,
            None => write!(f, ", dynamic ndim={}", self.ndim())?,
        }
        Ok(())
    }
}

/// Format the array using `LowerExp` and apply the formatting parameters used
/// to each element.
///
/// The array is shown in multiline style.
impl<'a, A: fmt::LowerExp, S, D: Dimension> fmt::LowerExp for ArrayBase<S, D>
where
    S: Data<Elem = A>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_array(self, f, <_>::fmt, PRINT_ELEMENTS_LIMIT)
    }
}

/// Format the array using `UpperExp` and apply the formatting parameters used
/// to each element.
///
/// The array is shown in multiline style.
impl<'a, A: fmt::UpperExp, S, D: Dimension> fmt::UpperExp for ArrayBase<S, D>
where
    S: Data<Elem = A>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_array(self, f, <_>::fmt, PRINT_ELEMENTS_LIMIT)
    }
}
/// Format the array using `LowerHex` and apply the formatting parameters used
/// to each element.
///
/// The array is shown in multiline style.
impl<'a, A: fmt::LowerHex, S, D: Dimension> fmt::LowerHex for ArrayBase<S, D>
where
    S: Data<Elem = A>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_array(self, f, <_>::fmt, PRINT_ELEMENTS_LIMIT)
    }
}

/// Format the array using `Binary` and apply the formatting parameters used
/// to each element.
///
/// The array is shown in multiline style.
impl<'a, A: fmt::Binary, S, D: Dimension> fmt::Binary for ArrayBase<S, D>
where
    S: Data<Elem = A>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_array(self, f, <_>::fmt, PRINT_ELEMENTS_LIMIT)
    }
}

#[cfg(test)]
mod formatting_with_omit {
    use super::*;
    use crate::prelude::*;

    fn print_output_diff(expected: &str, actual: &str) {
        println!("Expected output:\n{}\nActual output:\n{}", expected, actual);
    }

    #[test]
    fn empty_arrays() {
        let a: Array2<u32> = arr2(&[[], []]);
        let actual_output = format!("{}", a);
        let expected_output = String::from("[[]]");
        print_output_diff(&expected_output, &actual_output);
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn zero_length_axes() {
        let a = Array3::<f32>::zeros((3, 0, 4));
        let actual_output = format!("{}", a);
        let expected_output = String::from("[[[]]]");
        print_output_diff(&expected_output, &actual_output);
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn dim_0() {
        let element = 12;
        let a = arr0(element);
        let actual_output = format!("{}", a);
        let expected_output = format!("{}", element);
        print_output_diff(&expected_output, &actual_output);
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn dim_1() {
        let overflow: usize = 5;
        let a = Array1::from_elem((PRINT_ELEMENTS_LIMIT * 2 + overflow,), 1);
        let mut expected_output = String::from("[");
        a.iter()
            .take(PRINT_ELEMENTS_LIMIT)
            .for_each(|elem| expected_output.push_str(format!("{}, ", elem).as_str()));
        expected_output.push_str("...");
        a.iter()
            .skip(PRINT_ELEMENTS_LIMIT + overflow)
            .for_each(|elem| expected_output.push_str(format!(", {}", elem).as_str()));
        expected_output.push(']');
        let actual_output = format!("{}", a);

        print_output_diff(&expected_output, &actual_output);
        assert_eq!(actual_output, expected_output);
    }

    #[test]
    fn dim_2_last_axis_overflow() {
        let overflow: usize = 3;
        let a = Array2::from_elem(
            (PRINT_ELEMENTS_LIMIT, PRINT_ELEMENTS_LIMIT * 2 + overflow),
            1,
        );
        let mut expected_output = String::from("[");

        for i in 0..PRINT_ELEMENTS_LIMIT {
            expected_output.push_str(format!("[{}", a[(i, 0)]).as_str());
            for j in 1..PRINT_ELEMENTS_LIMIT {
                expected_output.push_str(format!(", {}", a[(i, j)]).as_str());
            }
            expected_output.push_str(", ...");
            for j in PRINT_ELEMENTS_LIMIT + overflow..PRINT_ELEMENTS_LIMIT * 2 + overflow {
                expected_output.push_str(format!(", {}", a[(i, j)]).as_str());
            }
            expected_output.push_str(if i < PRINT_ELEMENTS_LIMIT - 1 {
                "],\n "
            } else {
                "]"
            });
        }
        expected_output.push(']');
        let actual_output = format!("{}", a);

        print_output_diff(&expected_output, &actual_output);
        assert_eq!(actual_output, expected_output);
    }

    #[test]
    fn dim_2_non_last_axis_overflow() {
        let overflow: usize = 5;
        let a = Array2::from_elem(
            (PRINT_ELEMENTS_LIMIT * 2 + overflow, PRINT_ELEMENTS_LIMIT),
            1,
        );
        let mut expected_output = String::from("[");

        for i in 0..PRINT_ELEMENTS_LIMIT {
            expected_output.push_str(format!("[{}", a[(i, 0)]).as_str());
            for j in 1..PRINT_ELEMENTS_LIMIT {
                expected_output.push_str(format!(", {}", a[(i, j)]).as_str());
            }
            expected_output.push_str("],\n ");
        }
        expected_output.push_str("...,\n ");
        for i in PRINT_ELEMENTS_LIMIT + overflow..PRINT_ELEMENTS_LIMIT * 2 + overflow {
            expected_output.push_str(format!("[{}", a[(i, 0)]).as_str());
            for j in 1..PRINT_ELEMENTS_LIMIT {
                expected_output.push_str(format!(", {}", a[(i, j)]).as_str());
            }
            expected_output.push_str(if i == PRINT_ELEMENTS_LIMIT * 2 + overflow - 1 {
                "]"
            } else {
                "],\n "
            });
        }
        expected_output.push(']');
        let actual_output = format!("{}", a);

        print_output_diff(&expected_output, &actual_output);
        assert_eq!(actual_output, expected_output);
    }

    #[test]
    fn dim_2_multi_directional_overflow() {
        let overflow: usize = 5;
        let a = Array2::from_elem(
            (
                PRINT_ELEMENTS_LIMIT * 2 + overflow,
                PRINT_ELEMENTS_LIMIT * 2 + overflow,
            ),
            1,
        );
        let mut expected_output = String::from("[");

        for i in 0..PRINT_ELEMENTS_LIMIT {
            expected_output.push_str(format!("[{}", a[(i, 0)]).as_str());
            for j in 1..PRINT_ELEMENTS_LIMIT {
                expected_output.push_str(format!(", {}", a[(i, j)]).as_str());
            }
            expected_output.push_str(", ...");
            for j in PRINT_ELEMENTS_LIMIT + overflow..PRINT_ELEMENTS_LIMIT * 2 + overflow {
                expected_output.push_str(format!(", {}", a[(i, j)]).as_str());
            }
            expected_output.push_str("],\n ");
        }
        expected_output.push_str("...,\n ");
        for i in PRINT_ELEMENTS_LIMIT + overflow..PRINT_ELEMENTS_LIMIT * 2 + overflow {
            expected_output.push_str(format!("[{}", a[(i, 0)]).as_str());
            for j in 1..PRINT_ELEMENTS_LIMIT {
                expected_output.push_str(format!(", {}", a[(i, j)]).as_str());
            }
            expected_output.push_str(", ...");
            for j in PRINT_ELEMENTS_LIMIT + overflow..PRINT_ELEMENTS_LIMIT * 2 + overflow {
                expected_output.push_str(format!(", {}", a[(i, j)]).as_str());
            }
            expected_output.push_str(if i == PRINT_ELEMENTS_LIMIT * 2 + overflow - 1 {
                "]"
            } else {
                "],\n "
            });
        }
        expected_output.push(']');
        let actual_output = format!("{}", a);

        print_output_diff(&expected_output, &actual_output);
        assert_eq!(actual_output, expected_output);
    }
}
