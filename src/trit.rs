use serde::{Deserialize, Serialize};

/// Ternary digit: -1, 0, or +1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Trit {
    MinusOne,
    Zero,
    PlusOne,
}

impl Trit {
    pub fn value(&self) -> i8 {
        match self {
            Trit::MinusOne => -1,
            Trit::Zero => 0,
            Trit::PlusOne => 1,
        }
    }

    pub fn from_value(v: i8) -> Self {
        match v {
            ..=-1 => Trit::MinusOne,
            0 => Trit::Zero,
            1.. => Trit::PlusOne,
        }
    }
}

impl From<i8> for Trit {
    fn from(v: i8) -> Self {
        Trit::from_value(v)
    }
}

impl From<Trit> for i8 {
    fn from(t: Trit) -> Self {
        t.value()
    }
}
