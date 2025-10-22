const DEFAULT_PAGE_SIZE: usize = 128;
const DEFAULT_PAGE_COUNT: usize = 4096;

pub type Address = usize;

#[derive(Clone)]
enum Page<const N: usize = DEFAULT_PAGE_SIZE> {
    Header {
        size: usize,
        next: Option<Address>,
        data: [u8; N],
    },
    Body {
        next: Option<Address>,
        data: [u8; N],
    },
    Empty,
}

impl<const N: usize> Page<N> {
    fn is_empty(&self) -> bool {
        matches!(self, Page::Empty)
    }
}

#[derive(Clone)]
pub struct VirtualMemory<const N: usize = DEFAULT_PAGE_SIZE> {
    pages: Vec<Page<N>>,
}

impl Default for VirtualMemory<DEFAULT_PAGE_SIZE> {
    fn default() -> Self {
        Self {
            pages: Vec::with_capacity(DEFAULT_PAGE_COUNT),
        }
    }
}

impl<const N: usize> VirtualMemory<N> {
    pub fn new() -> Self {
        Self { pages: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            pages: Vec::with_capacity(capacity),
        }
    }

    pub fn allocate<T: Into<Vec<u8>>>(&mut self, value: T) -> Address {
        let mut value: Vec<u8> = value.into();
        let value_size = value.len();

        let padding = (N - (value.len() % N)) % N;
        if padding > 0 {
            value.resize(value.len() + padding, 0);
        }

        let value_page_count = value.len() / N;
        let empty_pages = self.get_empty_pages(value_page_count);

        // populate pages
        let mut address = 0;
        for (i, (data, page_index)) in value.chunks(N).zip(empty_pages.clone()).enumerate() {
            let Ok(data): Result<[u8; N], _> = data.try_into() else {
                panic!("VirtualMemory: Invalid data size.")
            };

            let page = match i {
                0 => {
                    address = page_index;
                    Page::Header {
                        size: value_size,
                        next: empty_pages.get(i + 1).cloned(),
                        data: data,
                    }
                },
                _ => {
                    Page::Body {
                        next: empty_pages.get(i + 1).cloned(),
                        data: data,
                    }
                },
            };

            self.pages[page_index] = page;
        }

        address
    }

    pub fn deallocate(&mut self, address: Address) {
        let pages = self.get_filled_pages(address);
        for page in pages {
            self.pages[page] = Page::Empty;
        }
    }

    pub fn deref(&self, address: Address) -> Vec<u8> {
        let pages = self.get_filled_pages(address);

        let mut data_len: Option<usize> = None;
        let mut bytes = Vec::with_capacity(pages.len() * N);
        for page in pages {
            match &self.pages[page] {
                Page::Header { size, data, .. } => {
                    if data_len.is_some() {
                        panic!("VirtualMemory: Encountered multiple headers reading data.");
                    }

                    data_len = Some(*size);
                    bytes.extend(data);
                },
                Page::Body { data, .. } => {
                    bytes.extend(data);
                },
                Page::Empty => break,
            }
        }

        match data_len {
            None => Vec::new(),
            Some(data_len) => {
                bytes.truncate(data_len);
                bytes
            },
        }
    }

    fn get_empty_pages(&mut self, count: usize) -> Vec<Address> {
        let (_, mut empty_pages): (usize, Vec<usize>) = self.pages.iter().enumerate().fold(
            (0, Vec::new()),
            |(pages, mut page_indices), (i, page)| {
                if pages >= count {
                    (pages, page_indices)
                } else {
                    match page {
                        Page::Empty => {
                            page_indices.push(i);
                            (pages + 1, page_indices)
                        },
                        _ => (pages, page_indices),
                    }
                }
            },
        );

        if empty_pages.len() < count {
            let diff = count - empty_pages.len();
            self.pages.extend(vec![Page::Empty; diff]);

            let new_pages = self.pages.len() - diff..self.pages.len();
            empty_pages.extend(new_pages);
        }

        empty_pages
    }

    fn get_filled_pages(&self, start: Address) -> Vec<Address> {
        let mut addresses = vec![start];
        let mut cursor = self.pages.get(start);
        while let Some(page) = cursor {
            match page {
                Page::Empty => break,
                Page::Header { next, .. } | Page::Body { next, .. } => {
                    match next {
                        None => break,
                        Some(address) => {
                            addresses.push(*address);
                            cursor = self.pages.get(*address);
                        },
                    }
                },
            }
        }

        addresses
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PAGE_SIZE: usize = 16;

    #[test]
    fn test_allocate_full_single_page() -> Result<(), &'static str> {
        let mut mem: VirtualMemory<PAGE_SIZE> = VirtualMemory::with_capacity(1);

        let inserted = String::from("sixteencharacter");
        let addr = mem.allocate(inserted.clone());

        assert_eq!(1, mem.pages.len());
        assert_eq!(0, addr);

        let page = mem.pages.get(addr).ok_or("page not found")?;
        let Page::Header { size, next, data } = page else {
            return Err("expected header page");
        };
        assert_eq!(inserted.len(), *size);
        assert!(next.is_none());
        assert_eq!(inserted.as_bytes(), data);

        Ok(())
    }

    #[test]
    fn test_allocate_partial_single_page() -> Result<(), &'static str> {
        let mut mem: VirtualMemory<PAGE_SIZE> = VirtualMemory::with_capacity(1);

        let inserted = String::from("partial");
        let addr = mem.allocate(inserted.clone());

        assert_eq!(1, mem.pages.len());
        assert_eq!(0, addr);

        let page = mem.pages.get(addr).ok_or("page not found")?;
        let Page::Header { size, next, data } = page else {
            return Err("expected header page");
        };
        assert_eq!(inserted.len(), *size);
        assert!(next.is_none());
        assert_eq!(inserted.as_bytes(), &data[0..*size]);

        Ok(())
    }

    #[test]
    fn test_allocate_full_single_non_start_page() -> Result<(), &'static str> {
        let mut mem: VirtualMemory<PAGE_SIZE> = VirtualMemory::with_capacity(2);

        let _ = mem.allocate("start");

        let inserted = String::from("sixteencharacter");
        let addr = mem.allocate(inserted.clone());

        assert_eq!(2, mem.pages.len());
        assert_eq!(1, addr);

        let page = mem.pages.get(addr).ok_or("page not found")?;
        let Page::Header { size, next, data } = page else {
            return Err("expected header page");
        };
        assert_eq!(inserted.len(), *size);
        assert!(next.is_none());
        assert_eq!(inserted.as_bytes(), data);

        Ok(())
    }

    #[test]
    fn test_allocate_full_multiple_continuous_pages() -> Result<(), &'static str> {
        let mut mem: VirtualMemory<PAGE_SIZE> = VirtualMemory::with_capacity(2);

        let first = String::from("sixteencharacter");
        let second = first.clone();
        let inserted = first.clone() + &second;
        let addr = mem.allocate(first.clone() + &second);

        assert_eq!(2, mem.pages.len());
        assert_eq!(0, addr);

        let page = mem.pages.get(addr).ok_or("page not found")?;
        let Page::Header { size, next, data } = page else {
            return Err("expected header page");
        };
        assert_eq!(inserted.len(), *size);
        assert_eq!(Some(1_usize), *next);
        assert_eq!(first.as_bytes(), data);

        let addr = next.ok_or("invalid next address")?;
        let page = mem.pages.get(addr).ok_or("page not found")?;
        let Page::Body { next, data } = page else {
            return Err("expected body page");
        };
        assert!(next.is_none());
        assert_eq!(second.as_bytes(), data);

        Ok(())
    }

    #[test]
    fn test_allocate_partial_multiple_continuous_pages() -> Result<(), &'static str> {
        let mut mem: VirtualMemory<PAGE_SIZE> = VirtualMemory::with_capacity(2);

        let first = String::from("sixteencharacter");
        let second = String::from("sixteen");
        let inserted = first.clone() + &second;
        let addr = mem.allocate(inserted.clone());

        assert_eq!(2, mem.pages.len());
        assert_eq!(0, addr);

        let page = mem.pages.get(addr).ok_or("page not found")?;
        let Page::Header { size, next, data } = page else {
            return Err("expected header page");
        };
        assert_eq!(inserted.len(), *size);
        assert_eq!(Some(1_usize), *next);
        assert_eq!(first.as_bytes(), data);

        let addr = next.ok_or("invalid next address")?;
        let page = mem.pages.get(addr).ok_or("page not found")?;
        let Page::Body { next, data } = page else {
            return Err("expected body page");
        };
        assert!(next.is_none());
        assert_eq!(second.as_bytes(), &data[0..*size - PAGE_SIZE]);

        Ok(())
    }

    #[test]
    fn test_allocate_partial_multiple_segmented_pages() -> Result<(), &'static str> {
        let mut mem: VirtualMemory<PAGE_SIZE> = VirtualMemory::with_capacity(3);

        let start = mem.allocate("start");
        _ = mem.allocate("middle");
        mem.deallocate(start);

        let first = String::from("sixteencharacter");
        let second = String::from("sixteen");
        let inserted = first.clone() + &second;
        let addr = mem.allocate(inserted.clone());

        assert_eq!(3, mem.pages.len());
        assert_eq!(0, addr);

        let page = mem.pages.get(addr).ok_or("page not found")?;
        let Page::Header { size, next, data } = page else {
            return Err("expected header page");
        };
        assert_eq!(inserted.len(), *size);
        assert_eq!(Some(2_usize), *next);
        assert_eq!(first.as_bytes(), data);

        let addr = next.ok_or("invalid next address")?;
        let page = mem.pages.get(addr).ok_or("page not found")?;
        let Page::Body { next, data } = page else {
            return Err("expected body page");
        };
        assert!(next.is_none());
        assert_eq!(second.as_bytes(), &data[0..*size - PAGE_SIZE]);

        Ok(())
    }

    #[test]
    fn test_deallocate_single_page() -> Result<(), &'static str> {
        let mut mem: VirtualMemory<PAGE_SIZE> = VirtualMemory::with_capacity(2);
        let addr = mem.allocate("sixteencharacter");

        mem.deallocate(addr);
        assert_eq!(1, mem.pages.len());
        assert!(
            mem.pages.get(addr).is_some_and(|p| p.is_empty()),
            "expected page to be empty"
        );

        Ok(())
    }

    #[test]
    fn test_deref_single_page() {
        let mut mem: VirtualMemory<PAGE_SIZE> = VirtualMemory::with_capacity(2);

        let inserted = String::from("sixteencharacter");
        let addr = mem.allocate(inserted.clone());

        let data = mem.deref(addr);
        assert_eq!(inserted.as_bytes(), data);
    }

    #[test]
    fn test_deref_multiple_continuous_pages() {
        let mut mem: VirtualMemory<PAGE_SIZE> = VirtualMemory::with_capacity(2);

        let inserted = String::from("sixteencharactersixteencharacter");
        let addr = mem.allocate(inserted.clone());

        let data = mem.deref(addr);
        assert_eq!(inserted.as_bytes(), data);
    }

    #[test]
    fn test_deref_multiple_segmented_pages() {
        let mut mem: VirtualMemory<PAGE_SIZE> = VirtualMemory::with_capacity(2);

        let start = mem.allocate("start");
        _ = mem.allocate("middle");
        mem.deallocate(start);

        let inserted = String::from("sixteencharacter");
        let addr = mem.allocate(inserted.clone());

        let data = mem.deref(addr);
        assert_eq!(inserted.as_bytes(), data);
    }
}
