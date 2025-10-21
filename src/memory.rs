const DEFAULT_PAGE_SIZE: usize = 128;
const DEFAULT_PAGE_COUNT: usize = 4096;

type Address = usize;

#[derive(Clone)]
struct HeaderPage<const N: usize> {
    size: usize,
    next: Option<Address>,
    data: [u8; N],
}

#[derive(Clone)]
struct BodyPage<const N: usize> {
    next: Option<Address>,
    data: [u8; N],
}

#[derive(Clone)]
enum Page<const N: usize> {
    Header(HeaderPage<N>),
    Body(BodyPage<N>),
    Empty,
}

pub struct VirtualMemory<const N: usize> {
    pages: Vec<Page<N>>,
}

impl Default for VirtualMemory<DEFAULT_PAGE_SIZE> {
    fn default() -> Self {
        Self { pages: Vec::new() }
    }
}

impl<const N: usize> VirtualMemory<N> {
    pub fn new() -> Self {
        Self { pages: Vec::new() }
    }

    pub fn alloc<T: Into<Vec<u8>>>(&mut self, value: T) -> Address {
        let value: Vec<u8> = value.into();
        let value_page_count = value.len().div_ceil(N);

        if value_page_count == 0 {
            panic!("VirtualMemory: Cannot allocate zero bytes.");
        }

        // acquire needed pages
        let (_, mut empty_pages): (usize, Vec<usize>) = self.pages.iter().enumerate().fold(
            (0, Vec::new()),
            |(acquired_pages, mut page_indices), (i, page)| {
                if acquired_pages >= value_page_count {
                    (acquired_pages, page_indices)
                } else {
                    match page {
                        Page::Empty => {
                            page_indices.push(i);
                            (acquired_pages + 1, page_indices)
                        },
                        _ => (acquired_pages, page_indices),
                    }
                }
            },
        );

        if empty_pages.len() < value_page_count {
            let diff = value_page_count - empty_pages.len();
            self.pages.extend(vec![Page::Empty; diff]);

            let new_pages = self.pages.len() - diff..self.pages.len();
            empty_pages.extend(new_pages);
        }

        // populate acquired pages
        let mut address = 0;
        for (i, (data, page_index)) in value.chunks(N).zip(empty_pages.clone()).enumerate() {
            let Ok(data): Result<[u8; N], _> = data.try_into() else {
                panic!("VirtualMemory: Invalid data size.")
            };

            let page = match i {
                0 => {
                    address = page_index;
                    Page::Header(HeaderPage {
                        size: value.len(),
                        next: empty_pages.get(i + 1).cloned(),
                        data: data,
                    })
                },
                _ => Page::Empty,
            };

            self.pages[page_index] = page;
        }

        address
    }
}
