pub trait True {}
macro_rules! implement_true {
    ($target_type:tt for $($value:expr),+) => {$(impl True for $target_type<$value>{})+};
}

pub struct IsPowerOfTwo<const N: usize>;
implement_true!(IsPowerOfTwo for 2,4,8,16,32,64,128,256,512,1024,2048,5096,10192);