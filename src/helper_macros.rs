macro_rules! _impl_partial_eq {
    ($($t:ty);+) => {
        $(
            impl PartialEq for $t {
            fn eq(&self, other: &Self) -> bool {
                self.id == other.id // functions are only distinguished by their ids
            }
        })*
    };
}

macro_rules! _impl_hash {
    ($($t:ty);+) => {
        $(
            impl Hash for $t {
                fn hash<H: Hasher>(&self, state: &mut H) {
                    self.id.hash(state);
                } 
        })*
    };
}

macro_rules! _impl_ord {
    ($($t:ty);+) => {
        $(
            impl Ord for $t {
                fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                    self.id.cmp(&other.id)
                }
        })*
    };
}

macro_rules! _impl_partial_ord {
    ($($t:ty);+) => {
        $(
            impl PartialOrd for $t {
                fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                    self.id.partial_cmp(&other.id) 
                }
        })*
    };
}

#[macro_export]
macro_rules! match_or_continue {
    ($func:expr, $msg:expr) => {
        match $func {
            Ok(val) => val,
            Err(_) => {
                eprintln!("{}", $msg);
                continue;
            }
        }   
    };
}

pub use match_or_continue;