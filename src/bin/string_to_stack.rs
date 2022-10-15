use std::str::from_utf8;

fn main() {
    let mut stack: Vec<i32> = vec![];
    let msg = "∆hellø, crim€s!∆";
    let bytes = msg.as_bytes();
    let valid_bytes_len = bytes.len();
    let chonky_boytes = bytes.chunks_exact(4);
    let remainder = chonky_boytes.remainder();
    for item in chonky_boytes {
        let i = i32::from_le_bytes(<[u8; 4]>::try_from(item).expect("unreachable"));
        stack.push(i);
    }
    let mut remaining_chonk = [0u8; 4];
    remaining_chonk[0..remainder.len()].copy_from_slice(remainder);
    stack.push(i32::from_le_bytes(remaining_chonk));
    stack.push(valid_bytes_len as i32);

    // restore!
    let string_bytes_len = stack.pop().unwrap() as usize;
    let stack_items_len = 1 + (string_bytes_len >> 2);
    let stack_slice = stack.as_slice();
    let string_start = stack.len() - stack_items_len;
    let almost_string_stack = &stack_slice[string_start..][..stack_items_len];

    // TODO bytemuck or whatever
    let string_slice = unsafe {
        core::slice::from_raw_parts(almost_string_stack.as_ptr() as *const u8, string_bytes_len)
    };

    let s = from_utf8(string_slice);
    match s {
        Ok(s) => println!("yay: {s}"),
        Err(e) => eprintln!("oh noes, {e:?}"),
    }
}
