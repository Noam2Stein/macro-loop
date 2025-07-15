use derive_quote_to_tokens::ToTokens;
use syn::{RangeLimits, Token, parse::Parse, spanned::Spanned};

#[derive(Clone, Copy, ToTokens)]
pub enum BinOp {
    Add(Token![+]),
    Sub(Token![-]),
    Mul(Token![*]),
    Div(Token![/]),
    Rem(Token![%]),

    And(Token![&]),
    Or(Token![|]),
    Xor(Token![^]),
    Shl(Token![<<]),
    Shr(Token![>>]),

    Range(Token![..]),
    RangeInclusive(Token![..=]),
}

#[derive(Clone, Copy, ToTokens)]
pub enum UnOp {
    Neg(Token![-]),

    Not(Token![!]),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BinOpLvl {
    ShlShr,
    MulDivRem,
    AddSub,
    And,
    Xor,
    Or,
    Range,
}

impl BinOp {
    pub fn option_parse(input: syn::parse::ParseStream) -> Option<Self> {
        macro_rules! option_parse {
            ($punct:tt => $variant:ident) => {
                if let Some(inner) = input.parse::<Option<Token![$punct]>>().unwrap() {
                    return Some(Self::$variant(inner));
                }
            };
        }

        if let Ok(limts) = RangeLimits::parse(input) {
            match limts {
                RangeLimits::Closed(op) => return Some(Self::RangeInclusive(op)),
                RangeLimits::HalfOpen(op) => return Some(Self::Range(op)),
            }
        }

        option_parse!(+ => Add);
        option_parse!(- => Sub);
        option_parse!(* => Mul);
        option_parse!(/ => Div);
        option_parse!(% => Rem);

        option_parse!(& => And);
        option_parse!(| => Or);
        option_parse!(^ => Xor);
        option_parse!(<< => Shl);
        option_parse!(>> => Shr);

        None
    }

    pub fn lvl(&self) -> BinOpLvl {
        match self {
            Self::Mul(_) | Self::Div(_) | Self::Rem(_) => BinOpLvl::MulDivRem,
            Self::Add(_) | Self::Sub(_) => BinOpLvl::AddSub,
            Self::And(_) => BinOpLvl::And,
            Self::Xor(_) => BinOpLvl::Xor,
            Self::Or(_) => BinOpLvl::Or,
            Self::Shl(_) | Self::Shr(_) => BinOpLvl::ShlShr,
            Self::Range(_) | Self::RangeInclusive(_) => BinOpLvl::Range,
        }
    }
}

impl UnOp {
    pub fn option_parse(input: syn::parse::ParseStream) -> Option<Self> {
        macro_rules! option_parse {
            ($punct:tt => $variant:ident) => {
                option_parse!($punct => $variant($punct))
            };

            ($punct:tt => $variant:ident($variant_punct:tt)) => {
                if let Some(inner) = input.parse::<Option<Token![$punct]>>().unwrap() {
                    return Some(Self::$variant(Token![$variant_punct](inner.span())));
                }
            };
        }

        option_parse!(- => Neg);

        option_parse!(! => Not);

        None
    }
}
