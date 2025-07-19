use proc_macro2::Span;
use syn::{Error, LitInt, Token};

use super::*;

impl<'v> ValueRef<'v> {
    pub fn method(self, method: &IdentStr, inputs: &[ValueRef<'v>]) -> syn::Result<ValueRef<'v>> {
        Ok(match method.str() {
            "enumerate" => Self::enumerate_method(self, method.span(), inputs)?,
            "index" => Self::index_method(&self, method.span(), inputs)?,
            "min" => Self::min_method(self, method.span(), inputs)?,
            "max" => Self::max_method(self, method.span(), inputs)?,
            "clamp" => Self::clamp_method(self, method.span(), inputs)?,

            _ => return Err(Error::new_spanned(&method, "Unknown method")),
        })
    }

    fn min_method(self, span: Span, inputs: &[ValueRef<'v>]) -> syn::Result<Self> {
        let &[ref other] = match inputs.try_into() {
            Ok(inputs) => inputs,
            _ => return Err(Error::new(span, "expected 1 argument")),
        };

        let lt_other = self.bin_op(BinOp::Lt(Token![<](span)), &other)?;
        let lt_other = match lt_other {
            Value::Bool(b) => b.value,
            _ => unreachable!(),
        };

        Ok(if lt_other { self } else { other.clone() })
    }

    fn max_method(self, span: Span, inputs: &[ValueRef<'v>]) -> syn::Result<Self> {
        let &[ref other] = match inputs.try_into() {
            Ok(inputs) => inputs,
            _ => return Err(Error::new(span, "expected 1 argument")),
        };

        let gt_other = self.bin_op(BinOp::Gt(Token![>](span)), &other)?;
        let gt_other = match gt_other {
            Value::Bool(b) => b.value,
            _ => unreachable!(),
        };

        Ok(if gt_other { self } else { other.clone() })
    }

    fn clamp_method(self, span: Span, inputs: &[ValueRef<'v>]) -> syn::Result<Self> {
        let &[ref min, ref max] = match inputs.try_into() {
            Ok(inputs) => inputs,
            _ => return Err(Error::new(span, "expected 2 arguments")),
        };

        let lt_min = self.bin_op(BinOp::Lt(Token![<](span)), &min)?;
        let lt_min = match lt_min {
            Value::Bool(b) => b.value,
            _ => unreachable!(),
        };

        let gt_max = self.bin_op(BinOp::Gt(Token![>](span)), &max)?;
        let gt_max = match gt_max {
            Value::Bool(b) => b.value,
            _ => unreachable!(),
        };

        Ok(if lt_min {
            min.clone()
        } else if gt_max {
            max.clone()
        } else {
            self
        })
    }

    fn enumerate_method(self, span: Span, inputs: &[ValueRef<'v>]) -> syn::Result<Self> {
        let &[] = match inputs.try_into() {
            Ok(inputs) => inputs,
            _ => return Err(Error::new(span, "expected 0 arguments")),
        };

        Ok(match &*self {
            Value::List(self_) => Self::Owned(Value::List(ValueList {
                span: self_.span,
                items: self_
                    .items
                    .iter()
                    .enumerate()
                    .map(|(idx, item)| {
                        Self::Owned(Value::List(ValueList {
                            span,
                            items: vec![
                                Self::Owned(Value::Int(LitInt::new(&idx.to_string(), span))),
                                item.clone(),
                            ],
                        }))
                    })
                    .collect(),
            })),

            _ => return Err(Error::new(span, "expected a list")),
        })
    }

    fn index_method(&self, span: Span, inputs: &[ValueRef<'v>]) -> syn::Result<ValueRef<'v>> {
        let &[ref idx] = match inputs.try_into() {
            Ok(inputs) => inputs,
            _ => return Err(Error::new(span, "expected 1 argument")),
        };

        match &**idx {
            Value::Int(idx) => {
                let idx = idx.base10_parse::<usize>().unwrap();

                Ok(self.index_cloned(idx, span)?)
            }

            Value::List(indicies) => {
                return Ok(Self::Owned(Value::List(ValueList {
                    span: indicies.span,
                    items: indicies
                        .clone()
                        .items
                        .into_iter()
                        .map(|idx| Self::index_method(self, span, &[idx]))
                        .collect::<syn::Result<_>>()?,
                })));
            }

            input => return Err(Error::new_spanned(input, "expected an int")),
        }
    }
}
