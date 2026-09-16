/// BUFFER_SIZE defines the maximum number of nested matches
/// that can end at the same character.
/// Must be 2^n for bitwise masking to work.
const BUFFER_SIZE: usize = 256; // Overkill value for the most realistic tasks.

pub(crate) struct RingBuffer<T> {
    pending_matches: [std::mem::MaybeUninit<T>; BUFFER_SIZE],
    head: usize,
    count: usize,
}
impl<T> RingBuffer<T> {
	pub(crate) fn new() -> Self {
        Self {
            pending_matches: [const {std::mem::MaybeUninit::uninit()}; BUFFER_SIZE],
            head: 0,
            count: 0,
        }
    }
	
	#[inline(always)]
	pub(crate) fn push_back(&mut self, value: T) {
		if self.count < BUFFER_SIZE {
            // SAFETY: head and count are less that BUFFER_SIZE always,
            // and BUFFER_SIZE < usize::MAX && BUFFER_SIZE > 0
			let index = unsafe { (self.head.unchecked_add(self.count)) & (BUFFER_SIZE.unchecked_sub(1)) };

			// SAFETY:
            // 1. `index` is guaranteed to be within [0, BUFFER_SIZE-1]
			//    due to bitwise mask with (BUFFER_SIZE-1).
            // 2. We are writing to a valid memory location of type MaybeUninit<T>.
            unsafe { self.pending_matches.get_unchecked_mut(index).as_mut_ptr().write(value); }

			// SAFETY: head is less that BUFFER_SIZE always, and BUFFER_SIZE < usize::MAX
            self.count = unsafe { self.count.unchecked_add(1) };
		}
		else {
			let index = self.head;

			// SAFETY:
            // 1. `index` is within bounds [0, BUFFER_SIZE-1].
            // 2. Since count == BUFFER_SIZE, the slot at `head` contains a fully initialized T.
            // 3. We drop the old value before overwriting it to prevent memory leaks.
			unsafe {
				let ptr = self.pending_matches.get_unchecked_mut(index).as_mut_ptr();
				std::ptr::drop_in_place(ptr);
				std::ptr::write(ptr, value);
			}

            // SAFETY: head is less that BUFFER_SIZE always,
            // and BUFFER_SIZE < usize::MAX && BUFFER_SIZE > 0
            self.head = unsafe { (self.head.unchecked_add(1)) & (BUFFER_SIZE.unchecked_sub(1)) };
		}
	}
	
    #[inline(always)]
    pub(crate) fn pop_front(&mut self) -> Option<T> {
        if self.count == 0 { None }
		else {
            let index = self.head;

			// SAFETY:
            // 1. `index` is within bounds [0, BUFFER_SIZE-1].
            // 2. Since count > 0, the element at `head` has been initialized by a previous push.
            // 3. We use `ptr::read` to move the value out without dropping the (now uninitialized) slot.
            let value = unsafe { std::ptr::read(self.pending_matches.get_unchecked(index).as_ptr()) };

            // SAFETY: head is less that BUFFER_SIZE always,
            // and BUFFER_SIZE < usize::MAX && BUFFER_SIZE > 0
            self.head = unsafe { (self.head.unchecked_add(1)) & (BUFFER_SIZE.unchecked_sub(1)) };

            // SAFETY: self.count > 0
            self.count = unsafe { self.count.unchecked_sub(1) };

            Some(value)
        }
    }
}

impl<T> Drop for RingBuffer<T> {
    fn drop(&mut self) {
        let mut i = 0;
        while i < self.count {
            // SAFETY: head and count are less that BUFFER_SIZE always,
            // and BUFFER_SIZE < usize::MAX && BUFFER_SIZE > 0
            let index = unsafe { (self.head.unchecked_add(i)) & (BUFFER_SIZE.unchecked_sub(1)) };

			// SAFETY:
            // 1. `index` is within bounds [0, BUFFER_SIZE-1].
            // 2. Elements from `head` to `head + count - 1` are guaranteed to be initialized.
            unsafe { 
				let ptr = self.pending_matches.get_unchecked_mut(index).as_mut_ptr();
				std::ptr::drop_in_place(ptr);
			}

            // SAFETY: i < self.count < BUFFER_SIZE < usize::MAX always
            i = unsafe { i.unchecked_add(1) };
        }
    }
}


#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
	use super::*;
	
	#[test]
	fn test_ring_buffer1() {
		let mut buf = RingBuffer::new();
		buf.push_back("string1".to_owned());
		buf.push_back("string2".to_owned());
		
		assert_eq!(buf.count, 2);
		
		let s1 = buf.pop_front();
		assert_eq!(s1, Some("string1".to_owned()));
		
		let s2 = buf.pop_front();
		assert_eq!(s2, Some("string2".to_owned()));

		let s3 = buf.pop_front();
		assert_eq!(s3, None);
		assert_eq!(buf.count, 0);
	}
	
	#[test]
    fn test_ring_buffer2() {
        let mut buf = RingBuffer::new();
        buf.push_back("a".to_string());
        buf.push_back("b".to_string());
        assert_eq!(buf.count, 2);

        for i in 0..BUFFER_SIZE + 10 {
            buf.push_back(i.to_string());
        }
        assert_eq!(buf.count, BUFFER_SIZE);

        let first = buf.pop_front().unwrap();
        assert_eq!(first, "10");
    }
}