use super::vm::{Cell, CellData, VM};

pub fn assert_similar(expected: f64, actual: CellData, decimals: u8) {
    let fac = 10f64.powf(decimals as _);
    let actual = (actual.to_num::<f64>() * fac).round() as i32;
    let expected = (expected * fac).round() as i32;
    assert_eq!(actual, expected);
}

// TODO maybe better wrap a Cow?
pub struct StackSlice<'a, T>(pub &'a [Cell<T>]);

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "use-std", derive(thiserror::Error))]
pub enum StackSliceError {
    #[cfg_attr(feature = "use-std", error("Over capacity"))]
    OverCapacity,
    #[cfg_attr(feature = "use-std", error("Invalid content"))]
    InvalidContent,
}

impl<'a, T, const N: usize> TryFrom<StackSlice<'a, T>> for heapless::Vec<u8, N> {
    type Error = StackSliceError;

    fn try_from(stack: StackSlice<'a, T>) -> Result<Self, Self::Error> {
        let stack = stack.0;
        let content_bytes_len: usize = stack
            .last()
            .ok_or(StackSliceError::InvalidContent)?
            .unwrap_raw() as usize;
        let content_len = stack.len() - 1;
        let content = &stack[0..content_len];
        let res = &mut [0u8; N];

        for (i, packed_bytes) in content
            .iter()
            .map(|elem| match elem {
                Cell::Raw(val) => Ok(val),
                _ => Err(StackSliceError::InvalidContent),
            })
            .enumerate()
        {
            // bale
            let packed_bytes = packed_bytes?;
            // TODO we can probably get a chunked iter over `res` and zip it?
            res[i * 4..][..4].copy_from_slice(&packed_bytes.to_le_bytes());
        }

        // TODO using `collect` would apparently be infallible hence more straightforward;
        // also we'll never be over capacity here,
        // instead the actual problem is overrunning the `res` slice above!
        heapless::Vec::from_slice(&res[..content_bytes_len])
            .map_err(|_| StackSliceError::OverCapacity)
    }
}

pub fn pack<'a, FFI: 'a>(slice: &'a [u8]) -> impl DoubleEndedIterator<Item = Cell<FFI>> + 'a {
    let len = slice.len();
    let packed = slice.chunks(4).into_iter().map(|chunk| {
        let mut dst = [0, 0, 0, 0];
        dst[0..chunk.len()].copy_from_slice(chunk);
        let number = i32::from_le_bytes(dst);
        Cell::Raw(number)
    });

    let other = Some(Cell::Raw(len as i32)).into_iter();
    packed.chain(other)
}

#[cfg(test)]
mod tests {
    use core::str::from_utf8;
    use std::error::Error;

    use super::*;
    use crate::forth::{
        runtime::{stud::TestRuntime, PureJSFFI},
        vm::{Cell, Op, Stack},
    };

    #[test]
    fn test_empty() -> Result<(), Box<dyn Error>> {
        let s = "";
        let stack: Vec<Cell<PureJSFFI>> = pack(s.as_bytes()).collect();
        let v: heapless::Vec<u8, 32> = StackSlice(&stack).try_into()?;
        let de = from_utf8(v.as_slice())?;
        assert_eq!(de, s);
        Ok(())
    }

    #[test]
    fn test_str() -> Result<(), Box<dyn Error>> {
        let s = "∆ohai∆";
        let stack: Vec<Cell<PureJSFFI>> = pack(s.as_bytes()).collect();
        let v: heapless::Vec<u8, 32> = StackSlice(&stack).try_into()?;
        let de = from_utf8(v.as_slice())?;
        assert_eq!(de, s);
        Ok(())
    }

    #[test]
    fn test_ffi() -> Result<(), Box<dyn Error>> {
        let s = "∆ohai∆";
        let mut stack: Vec<Cell<PureJSFFI>> = pack(s.as_bytes()).collect();
        stack.push(Cell::Op(Op::FFI(PureJSFFI::ConsoleLog)));
        dbg!(&stack);

        let mut vm = VM::new(
            Stack::from_slice(&stack).unwrap(),
            Default::default(),
            TestRuntime::new(),
        );
        vm.run().ok();
        let rt = vm.dismember();
        assert_eq!(Some(s), rt.last_log());
        Ok(())
    }
}
