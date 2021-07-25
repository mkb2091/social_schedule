macro_rules! create_range {
    ($x_range: ident, $x: ident) => {
        #[derive(Copy, Clone, Debug)]
        pub struct $x(usize);
        impl $x {
            pub const fn as_usize(self) -> usize {
                self.0
            }
        }
        #[derive(Copy, Clone, Debug)]
        pub struct $x_range {
            start: usize,
            end: usize,
        }
        impl $x_range {
            pub const fn new(start: usize, end: usize) -> Self {
                Self { start, end }
            }
            pub const fn skip(mut self, n: usize) -> Self {
                self.start += n;
                self
            }
            pub fn next(&mut self) -> Option<$x> {
                if self.start < self.end {
                    let result = self.start;
                    self.start += 1;
                    Some($x(result))
                } else {
                    None
                }
            }
            pub const fn convert_usize(&self, n: usize) -> Option<$x> {
                if n >= self.start && n < self.end {
                    Some($x(n))
                } else {
                    None
                }
            }
        }
    };
}

create_range!(RoundRange, Round);
create_range!(TableRange, Table);
