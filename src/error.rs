// Copyright (c) 2017 Brandon Thomas <bt@brand.io>
// Painting with Lasers
use std::sync::PoisonError;
use std::sync::RwLockWriteGuard;
use lase::Point;

#[derive(Debug)]
pub enum PaintError {
  ConcurrencyError,
  UnknownError,
}

impl<T> From<PoisonError<T>> for PaintError {
  fn from(_: PoisonError<T>) -> Self {
    PaintError::ConcurrencyError
  }
}
