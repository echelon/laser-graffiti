// Copyright (c) 2017 Brandon Thomas <bt@brand.io>
// Painting with Lasers
use std::sync::PoisonError;

#[derive(Debug)]
pub enum PaintError {
  ConcurrencyError,
}

impl<T> From<PoisonError<T>> for PaintError {
  fn from(_: PoisonError<T>) -> Self {
    PaintError::ConcurrencyError
  }
}
