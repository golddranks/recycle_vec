use recycle_vec::VecExt;

pub fn main() {
    let mut buf = Vec::with_capacity(100);
    buf.push(1_u16);
    {
        let mut buf2 = buf.recycle();
        buf2.push(1_u32);
        buf = buf2.recycle();
    }
    buf.push(1_u16);
}
