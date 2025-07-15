use derive_quote_to_tokens::ToTokens;
use syn::{RangeLimits, Token, parse::Parse, spanned::Spanned};

#[derive(Clone, Copy, ToTokens)]
pub enum BinOp {
    Add(Token![+]),
    Sub(Token![-]),
    Mul(Token![*]),
    Div(Token![/]),
    Rem(Token![%]),

    BitAnd(Token![&]),
    BitOr(Token![|]),
    BitXor(Token![^]),
    Shl(Token![<<]),
    Shr(Token![>>]),

    Eq(Token![==]),
    Ne(Token![!=]),
    Lt(Token![<]),
    Gt(Token![>]),
    Le(Token![<=]),
    Ge(Token![>=]),
    LogicalAnd(Token![&&]),
    LogicalOr(Token![||]),

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
    BitAnd,
    BitXor,
    BitOr,
    Range,
    Eq,
    LogicalAnd,
    LogicalOr,
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

        option_parse!(&& => LogicalAnd);
        option_parse!(|| => LogicalOr);

        option_parse!(== => Eq);
        option_parse!(!= => Ne);
        option_parse!(<= => Le);
        option_parse!(>= => Ge);

        option_parse!(+ => Add);
        option_parse!(- => Sub);
        option_parse!(* => Mul);
        option_parse!(/ => Div);
        option_parse!(% => Rem);

        option_parse!(& => BitAnd);
        option_parse!(| => BitOr);
        option_parse!(^ => BitXor);
        option_parse!(<< => Shl);
        option_parse!(>> => Shr);

        option_parse!(< => Lt);
        option_parse!(> => Gt);

        None
    }

    pub fn lvl(&self) -> BinOpLvl {
        match self {
            Self::Mul(_) | Self::Div(_) | Self::Rem(_) => BinOpLvl::MulDivRem,
            Self::Add(_) | Self::Sub(_) => BinOpLvl::AddSub,
            Self::BitAnd(_) => BinOpLvl::BitAnd,
            Self::BitXor(_) => BinOpLvl::BitXor,
            Self::BitOr(_) => BinOpLvl::BitOr,
            Self::Shl(_) | Self::Shr(_) => BinOpLvl::ShlShr,
            Self::Range(_) | Self::RangeInclusive(_) => BinOpLvl::Range,
            Self::Eq(_) | Self::Ne(_) | Self::Lt(_) | Self::Gt(_) | Self::Le(_) | Self::Ge(_) => {
                BinOpLvl::Eq
            }
            Self::LogicalAnd(_) => BinOpLvl::LogicalAnd,
            Self::LogicalOr(_) => BinOpLvl::LogicalOr,
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
