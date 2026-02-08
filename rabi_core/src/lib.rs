pub trait IntoRaw {
    type Output: RawRepr;

    fn into_raw(self) -> Raw<Self::Output>;
}

pub trait FromRaw {
    type Input: RawRepr;
    type Output;

    fn from_raw(raw: Raw<Self::Input>) -> Self::Output;
}

pub trait RawRepr {
    type Repr;
}

pub type Raw<T> = InnerRaw<<T as RawRepr>::Repr>;

#[repr(C)]
pub union InnerRaw<T> {
    pub value: std::mem::ManuallyDrop<T>,
    pub vec: std::mem::ManuallyDrop<RawVec<T>>,
    pub string: std::mem::ManuallyDrop<RawVec<u8>>,
}

#[repr(C)]
pub struct RawVec<T> {
    ptr: *mut T,
    len: usize,
    capacity: usize,
}

impl RawRepr for String {
    type Repr = String;
}

impl IntoRaw for String {
    type Output = String;

    fn into_raw(self) -> InnerRaw<Self::Output> {
        let (ptr, len, capacity) = self.into_raw_parts();
        InnerRaw {
            string: std::mem::ManuallyDrop::new(RawVec { ptr, len, capacity }),
        }
    }
}

impl FromRaw for String {
    type Input = String;
    type Output = Self;

    fn from_raw(raw: InnerRaw<Self::Input>) -> Self::Output {
        let string = unsafe {
            match raw {
                InnerRaw { string } => string,
            }
        };
        unsafe { Self::from_raw_parts(string.ptr, string.len, string.capacity) }
    }
}

impl<T> RawRepr for Vec<T>
where
    T: RawRepr,
{
    type Repr = Vec<<T as RawRepr>::Repr>;
}

impl<T> IntoRaw for Vec<T>
where
    T: IntoRaw<Output = <T as RawRepr>::Repr> + RawRepr,
    <T as RawRepr>::Repr: RawRepr<Repr = <T as RawRepr>::Repr>,
{
    type Output = Vec<<T as RawRepr>::Repr>;

    fn into_raw(self) -> Raw<Self::Output> {
        let data: Vec<InnerRaw<<T as RawRepr>::Repr>> = self.into_iter().map(|v| v.into_raw()).collect();
        let (ptr, len, capacity) = data.into_raw_parts();
        InnerRaw {
            vec: std::mem::ManuallyDrop::new(RawVec {
                ptr: ptr as *mut Vec<<T as RawRepr>::Repr>,
                len,
                capacity,
            }),
        }
    }
}

impl<T> FromRaw for Vec<T>
where
    T: FromRaw<Input = <T as RawRepr>::Repr, Output = T> + RawRepr,
    <T as RawRepr>::Repr: RawRepr<Repr = <T as RawRepr>::Repr>,
{
    type Input = Vec<<T as RawRepr>::Repr>;
    type Output = Vec<T>;

    fn from_raw(raw: Raw<Self::Input>) -> Self::Output {
        let vec = unsafe {
            match raw {
                InnerRaw { vec } => vec,
            }
        };
        let data = unsafe { Vec::from_raw_parts(vec.ptr as *mut InnerRaw<<T as RawRepr>::Repr>, vec.len, vec.capacity) };
        data.into_iter().map(|v| T::from_raw(v)).collect()
    }
}

macro_rules! impl_raw_primitive {
    ($ty:ty) => {
        impl RawRepr for $ty {
            type Repr = $ty;
        }

        impl IntoRaw for $ty {
            type Output = $ty;

            fn into_raw(self) -> InnerRaw<Self::Output> {
                InnerRaw {
                    value: std::mem::ManuallyDrop::new(self),
                }
            }
        }

        impl FromRaw for $ty {
            type Input = $ty;
            type Output = $ty;

            fn from_raw(raw: InnerRaw<Self::Input>) -> Self::Output {
                unsafe {
                    match raw {
                        InnerRaw { value } => *value,
                    }
                }
            }
        }
    };
}

impl_raw_primitive!(bool);
impl_raw_primitive!(char);
impl_raw_primitive!(u8);
impl_raw_primitive!(u16);
impl_raw_primitive!(u32);
impl_raw_primitive!(u64);
impl_raw_primitive!(u128);
impl_raw_primitive!(usize);
impl_raw_primitive!(i8);
impl_raw_primitive!(i16);
impl_raw_primitive!(i32);
impl_raw_primitive!(i64);
impl_raw_primitive!(i128);
impl_raw_primitive!(isize);
impl_raw_primitive!(f32);
impl_raw_primitive!(f64);
