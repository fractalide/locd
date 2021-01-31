use lazy_static::lazy_static;

lazy_static! {
    pub static ref HBFI_LUT: [u64; 256] = {
        let mut l = [0; 256];

        l[b'A' as usize] = 1;
        l[b'B' as usize] = 2;
        l[b'C' as usize] = 3;
        l[b'D' as usize] = 4;

        l
    };
}
