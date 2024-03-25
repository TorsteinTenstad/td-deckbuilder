use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumFlags<Enum: Clone + Into<usize>> {
    value: usize,
    _phantom: std::marker::PhantomData<Enum>,
}

impl<Enum: Clone + Into<usize>> EnumFlags<Enum> {
    pub fn new() -> Self {
        debug_assert!(std::mem::size_of::<Enum>() <= std::mem::size_of::<usize>());
        Self {
            value: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn set(&mut self, flag: &Enum) -> &Self {
        let enum_usize: usize = (*flag).clone().into();
        self.value |= 1 << enum_usize;
        self
    }

    pub fn unset(&mut self, flag: &Enum) -> &Self {
        let enum_usize: usize = (*flag).clone().into();
        self.value &= !(1 << enum_usize);
        self
    }

    pub fn is_set(&self, flag: &Enum) -> bool {
        let enum_usize: usize = (*flag).clone().into();
        self.value & (1 << enum_usize) != 0
    }

    pub fn all() -> Self {
        Self {
            value: !0,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Enum: Clone + Into<usize>> Default for EnumFlags<Enum> {
    fn default() -> Self {
        Self::new()
    }
}

#[macro_export]
macro_rules! flags {
    () => {
        EnumFlags::<_>::new()
    };

    ($flag:expr $(, $rest:expr)*) => {
        {
            let mut flags = EnumFlags::new();
            flags.set(&$flag);
            $(
                flags.set(&$rest);
            )*
            flags
        }
    };

    ($flag:expr $(, $rest:expr)*,) => {
        {
            let mut flags = EnumFlags::new();
            flags.set(&$flag);
            $(
                flags.set(&$rest);
            )*
            flags
        }
    };
}

pub use flags;
