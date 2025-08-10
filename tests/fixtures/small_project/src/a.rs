pub fn a() -> i32 { 1 }

pub mod inner {
    pub fn ia() -> i32 { super::a() + 1 }
}