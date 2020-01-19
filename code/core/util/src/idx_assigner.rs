use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Add;

// Traits

pub trait Index: Clone + Copy + PartialOrd + Debug {
  fn one() -> Self;

  fn is_zero(self) -> bool;

  fn into_usize(self) -> usize;

  fn add(self, rhs: Self) -> Self;

  fn overflowing_add(self, rhs: Self) -> (Self, bool);
}

pub trait Item: Default + Clone + Copy + Debug {
  type Idx: Index;

  fn new(index: Self::Idx) -> Self;

  fn into_idx(self) -> Self::Idx;
}

// Index allocator

#[derive(Default)]
pub struct IdxAssigner<I: Item<Idx=Idx>, Idx: Index = u32> {
  next_idx: Idx,
  _phantom: PhantomData<I>,
}

impl<I: Item<Idx=Idx>, Idx: Index> IdxAssigner<I, Idx> {
  pub fn new() -> Self {
    debug_assert!(I::default().into_idx().is_zero(), "BUG: index in default item {:?} is not zero", I::default());
    Self { next_idx: Idx::one(), _phantom: PhantomData::default() }
  }

  #[inline]
  pub fn exists(&self, item: I) -> bool {
    let idx = item.into_idx();
    !idx.is_zero() && idx < self.next_idx
  }

  #[inline]
  pub fn assign_item(&mut self) -> I {
    let (new_next_idx, overflow) = self.next_idx.overflowing_add(Idx::one());
    let item = I::new(self.next_idx);
    debug_assert!(!overflow, "ERR: cannot assign new item; overflow in index");
    self.next_idx = new_next_idx;
    item
  }

  pub fn assign_items(&mut self, count: Idx) -> Vec<I> {
    let (new_next_idx, overflow) = self.next_idx.overflowing_add(count);
    debug_assert!(!overflow, "ERR: cannot assign '{:?}' new items; overflow in index", count);
    // OPTO: version without allocation.
    let mut vec = Vec::with_capacity(count.into_usize());
    let mut next_idx = self.next_idx;
    for item in vec.iter_mut() {
      *item = I::new(next_idx);
      next_idx = next_idx.add(Idx::one());
    }
    self.next_idx = new_next_idx;
    vec
  }
}

// Implementations

macro_rules! uint_impl {
  ($ty:ty) => {
    impl Index for $ty {
      #[inline] fn one() -> Self { 1 }
      #[inline] fn is_zero(self) -> bool { self == 0 }
      #[inline] fn into_usize(self) -> usize { self as usize }
      #[inline] fn add(self, rhs: Self) -> Self { Add::add(self, rhs) }
      #[inline] fn overflowing_add(self, rhs: Self) -> (Self, bool) { self.overflowing_add(rhs) }
    }
  }
}
uint_impl!(u8);
uint_impl!(u16);
uint_impl!(u32);
uint_impl!(u64);
uint_impl!(u128);
