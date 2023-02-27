use std::marker::PhantomData;

pub struct ModifyHeader;

pub struct ModifyRows;

pub struct Table<T = ModifyHeader, const N: usize = 0> {
    headers: Vec<Header>,
    column_widths: Vec<usize>,
    rows: Vec<Vec<String>>,
    skip_header: bool,
    _pd: PhantomData<T>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Left,
    Right,
    Center,
}

pub struct Header {
    pub text: String,
    pub alignment: Alignment,
}

impl Into<Header> for &str {
    fn into(self) -> Header {
        Header {
            text: self.to_string(),
            alignment: Alignment::Left,
        }
    }
}

fn width(s: &str) -> usize {
    let bytes = strip_ansi_escapes::strip(&s).expect("Failed to strip escape sequences");
    let s = unsafe { std::str::from_utf8_unchecked(&bytes) };
    unicode_width::UnicodeWidthStr::width(s)
}

impl<T, const N: usize> Table<T, N> {
    pub fn update_widths(&mut self, row: &Row<N>) {
        for (w, cell) in self.column_widths.iter_mut()
            .zip(row.cells.iter()) {
            *w = std::cmp::max(*w, width(&cell));
        }
    }
}

impl<const N: usize> Table<ModifyHeader, N> {
    pub fn new() -> Self {
        Table {
            headers: Vec::new(),
            column_widths: Vec::new(),
            rows: Vec::new(),
            skip_header: false,
            _pd: PhantomData,
        }
    }

    pub fn header<H: Into<Header>>(mut self, header: H) -> Table<ModifyHeader, N> {
        let header = header.into();
        let width = width(&header.text);
        self.headers.push(header);
        self.column_widths.push(width);
        Table {
            headers: self.headers,
            column_widths: self.column_widths,
            rows: self.rows,
            skip_header: self.skip_header,
            _pd: PhantomData,
        }
    }

    pub fn row(mut self, row: Row<N>) -> Table<ModifyRows, N> {
        self.update_widths(&row);
        Table {
            headers: self.headers,
            column_widths: self.column_widths,
            rows: vec![row.cells],
            skip_header: self.skip_header,
            _pd: PhantomData,
        }
    }

    pub fn end_header(self) -> Table<ModifyRows, N> {
        Table {
            headers: self.headers,
            column_widths: self.column_widths,
            rows: Vec::new(),
            skip_header: self.skip_header,
            _pd: PhantomData,
        }
    }
}

impl<const N: usize> Table<ModifyRows, N> {
    pub fn row(mut self, row: Row<N>) -> Self {
        self.update_widths(&row);
        self.rows.push(row.cells);
        self
    }
}

fn format(s: &str, target_width: usize, alignment: Alignment) -> String {
    let width = width(s);
    let target_width = std::cmp::max(target_width, 8);
    let padding = target_width - width;
    match alignment {
        Alignment::Left => s.to_string() + &" ".repeat(padding),
        Alignment::Right => " ".repeat(padding) + s,
        Alignment::Center => {
            let left = padding / 2;
            let right = padding - left;
            " ".repeat(left) + s + &" ".repeat(right)
        }
    }
}

impl std::fmt::Display for Table<ModifyRows> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if !self.skip_header {
            for (header, width) in self.headers.iter()
                .zip(self.column_widths.iter()) {
                let header = format(&header.text, *width, header.alignment);
                write!(f, "{} ", header)?;
            }
            writeln!(f)?;
        }
        for row in self.rows.iter() {
            for (cell, width) in row.iter()
                .zip(self.column_widths.iter()) {
                let cell = format(cell, *width, Alignment::Left);
                write!(f, "{cell} ")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

pub struct Row<const N: usize> {
    cells: Vec<String>,
}

impl<const N: usize> Row<N> {
    pub fn new() -> Self {
        Row {
            cells: Vec::new(),
        }
    }

    pub fn cell(mut self, cell: &str) -> Row<N> {
        self.cells.push(cell.to_string());
        Row {
            cells: self.cells,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let table = Table::new()
            .header("Name")
            .header("Age")
            .row(Row::new().cell("Alice").cell("20"))
            .row(Row::new().cell("Bob").cell("30"));
        println!("{}", table);
        assert_eq!(table.to_string(),
                   "Name     Age      \n".to_owned() +
                       "Alice    20       \n" +
                       "Bob      30       \n");
    }
}