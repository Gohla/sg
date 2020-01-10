use std::collections::VecDeque;
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

pub trait Version: Default + Clone + Copy + PartialEq + Debug {
  fn one() -> Self;

  fn wrapping_add(self, rhs: Self) -> Self;
}

pub trait Item<Idx: Index, Ver: Version>: Default + Clone + Copy + Debug {
  fn new(index: Idx, version: Ver) -> Self;

  fn into_index(self) -> Idx;

  fn into_version(self) -> Ver;
}

// Index allocator

#[derive(Default)]
pub struct IdxAllocator<Idx: Index, Ver: Version, I: Item<Idx, Ver>> {
  slots: Vec<Ver>,
  num_slots: Idx,
  // Manually maintain number of slots as an u32 to prevent casting.
  free: VecDeque<Idx>,
  _phantom: PhantomData<I>,
}

impl<Idx: Index, Ver: Version, I: Item<Idx, Ver>> IdxAllocator<Idx, Ver, I> {
  pub fn new() -> Self {
    debug_assert!(I::default().into_index().is_zero(), "BUG: index in default item '{:?}' is not zero", I::default());
    debug_assert_eq!(Ver::default(), I::default().into_version(), "BUG: version in default item '{:?}' is not the default", I::default());
    let slots = {
      let mut slots = Vec::with_capacity(1);
      slots.push(Ver::default());
      slots
    };
    let num_slots = Idx::one();
    let free = VecDeque::with_capacity(Self::MIN_FREE_ITEMS);
    Self { slots, num_slots, free, _phantom: PhantomData::default() }
  }

  #[inline]
  pub fn exists(&self, item: I) -> bool {
    let idx = item.into_index();
    let ver = item.into_version();
    !idx.is_zero() && idx < self.num_slots && *unsafe { self.get_version_unchecked(idx) } == ver
  }

  #[inline]
  pub fn allocate_item(&mut self) -> I {
    self.alloc_item()
  }

  pub fn allocate_items(&mut self, count: Idx) -> Vec<I> {
    // OPTO: version without allocation.
    let mut vec = Vec::with_capacity(count.into_usize());
    for item in vec.iter_mut() {
      // OPTO: allocate all items in one go.
      *item = self.alloc_item();
    }
    vec
  }

  #[inline]
  pub fn deallocate_item(&mut self, item: I) {
    if self.exists(item) {
      self.dealloc_item(item.into_index())
    }
  }

  pub fn deallocate_items<Iter: IntoIterator<Item=I>>(&mut self, items: Iter) {
    for item in Iter::from(items) {
      if self.exists(item) {
        // OPTO: deallocate all items in one go.
        self.dealloc_item(item.into_index())
      }
    }
  }


  #[inline]
  fn alloc_item(&mut self) -> I {
    if self.free.len() > Self::MIN_FREE_ITEMS {
      let idx = self.free.pop_front().unwrap();
      let ver = unsafe { self.get_version_unchecked(idx) };
      I::new(idx, *ver)
    } else {
      let idx = self.num_slots;
      let (new_num_slots, overflow) = self.num_slots.overflowing_add(Idx::one());
      debug_assert!(!overflow, "ERR: cannot allocate new item; overflow in index");
      let ver = Ver::default();
      self.slots.push(ver);
      self.num_slots = new_num_slots;
      I::new(idx, ver)
    }
  }

  #[inline]
  fn dealloc_item(&mut self, idx: Idx) {
    debug_assert!(!idx.is_zero(), "BUG: item with zero index was given");
    debug_assert!(idx < self.num_slots, "BUG: out-of-bounds item index ('{:?}' >= item slot count '{:?}')", idx, self.num_slots);
    let ver = unsafe { self.get_version_unchecked_mut(idx) };
    *ver = ver.wrapping_add(Ver::one());
    self.free.push_back(idx);
  }

  #[inline]
  unsafe fn get_version_unchecked(&self, idx: Idx) -> &Ver {
    self.slots.get_unchecked(idx.into_usize())
  }

  #[inline]
  unsafe fn get_version_unchecked_mut(&mut self, idx: Idx) -> &mut Ver {
    self.slots.get_unchecked_mut(idx.into_usize())
  }

  const MIN_FREE_ITEMS: usize = 128;
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

    impl Version for $ty {
      #[inline] fn one() -> Self { 1 }
      #[inline] fn wrapping_add(self, rhs: Self) -> Self { self.wrapping_add(rhs) }
    }
  }
}
uint_impl!(u8);
uint_impl!(u16);
uint_impl!(u32);
uint_impl!(u64);
uint_impl!(u128);

impl<Idx: Index + From<T>, Ver: Version + From<T>, T: Default + Clone + Copy + Debug + From<(Idx, Ver)>> Item<Idx, Ver> for T {
  #[inline]
  fn new(idx: Idx, ver: Ver) -> Self { (idx, ver).into() }

  #[inline]
  fn into_index(self) -> Idx { self.into() }

  #[inline]
  fn into_version(self) -> Ver { self.into() }
}
