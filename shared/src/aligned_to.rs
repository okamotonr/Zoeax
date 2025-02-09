#[repr(C)] // guarantee 'bytes' comes after '_align'
pub struct AlignedTo<Align, Bytes: ?Sized> {
    pub _align: [Align; 0],
    pub bytes: Bytes,
}

