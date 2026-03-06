/// DBF (dBASE III/IV) and CSV file viewer

use std::path::Path;

#[derive(Debug, Clone)]
pub struct DbfField {
    pub name: String,
    pub field_type: char, // C=char, N=number, D=date, L=logical, M=memo
    pub length: usize,
}

#[derive(Debug, Clone)]
pub struct DbfData {
    pub fields: Vec<DbfField>,
    pub records: Vec<Vec<String>>,
    pub col_cursor: usize,
    pub row_cursor: usize,
    pub col_scroll: usize,
    pub row_scroll: usize,
    pub filename: String,
    pub source_type: SourceType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceType {
    Dbf,
    Csv,
}

impl DbfData {
    /// Open a .dbf file
    pub fn open(path: &Path) -> Result<Self, String> {
        let data = std::fs::read(path).map_err(|e| e.to_string())?;
        if data.len() < 32 {
            return Err("File too small — not a valid DBF".to_string());
        }

        // Header bytes
        let num_records_raw = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let header_size = u16::from_le_bytes([data[8], data[9]]) as usize;
        let record_size = u16::from_le_bytes([data[10], data[11]]) as usize;

        if header_size < 32 || record_size == 0 {
            return Err("Invalid DBF header".to_string());
        }

        // Parse field descriptors (32 bytes each, starting at byte 32, terminated by 0x0D)
        let mut fields: Vec<DbfField> = Vec::new();
        let mut offset = 32usize;
        while offset + 32 <= header_size.min(data.len()) {
            if data[offset] == 0x0D {
                break; // header terminator
            }
            let name_raw = &data[offset..offset + 11];
            let name = String::from_utf8_lossy(name_raw)
                .trim_end_matches('\0')
                .trim()
                .to_string();
            if name.is_empty() {
                break;
            }
            let field_type = data[offset + 11] as char;
            let length = data[offset + 16] as usize;
            fields.push(DbfField { name, field_type, length });
            offset += 32;
        }

        if fields.is_empty() {
            return Err("No fields found in DBF".to_string());
        }

        // Parse records
        let mut records: Vec<Vec<String>> = Vec::new();
        let num_records = num_records_raw as usize;
        for i in 0..num_records {
            let rec_offset = header_size + i * record_size;
            if rec_offset >= data.len() {
                break;
            }
            // Deletion marker: 0x20 = active, 0x2A = deleted
            if data[rec_offset] == 0x2A {
                continue;
            }

            let mut row: Vec<String> = Vec::new();
            let mut fld_offset = rec_offset + 1;
            for field in &fields {
                let end = (fld_offset + field.length).min(data.len());
                let raw = &data[fld_offset..end];
                let val = String::from_utf8_lossy(raw).trim().to_string();
                row.push(val);
                fld_offset += field.length;
            }
            records.push(row);
        }

        let filename = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        Ok(DbfData {
            fields,
            records,
            col_cursor: 0,
            row_cursor: 0,
            col_scroll: 0,
            row_scroll: 0,
            filename,
            source_type: SourceType::Dbf,
        })
    }

    /// Open a .csv file
    pub fn open_csv(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let mut lines = text.lines();
        let header_line = lines.next().ok_or("Empty CSV file")?;
        let field_names = parse_csv_row(header_line);
        let fields: Vec<DbfField> = field_names
            .into_iter()
            .map(|name| DbfField {
                name,
                field_type: 'C',
                length: 20,
            })
            .collect();
        let records: Vec<Vec<String>> = lines.map(parse_csv_row).collect();
        let filename = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        Ok(DbfData {
            fields,
            records,
            col_cursor: 0,
            row_cursor: 0,
            col_scroll: 0,
            row_scroll: 0,
            filename,
            source_type: SourceType::Csv,
        })
    }

    pub fn num_rows(&self) -> usize {
        self.records.len()
    }

    pub fn num_cols(&self) -> usize {
        self.fields.len()
    }

    pub fn col_width(&self, col: usize) -> usize {
        let name_len = self.fields.get(col).map_or(0, |f| f.name.len());
        let data_len = self
            .fields
            .get(col)
            .map_or(10, |f| f.length.max(f.name.len()).min(30));
        name_len.max(data_len).max(4).min(30)
    }

    pub fn cursor_up(&mut self) {
        if self.row_cursor > 0 {
            self.row_cursor -= 1;
            if self.row_cursor < self.row_scroll {
                self.row_scroll = self.row_cursor;
            }
        }
    }

    pub fn cursor_down(&mut self, visible_rows: usize) {
        if self.row_cursor + 1 < self.num_rows() {
            self.row_cursor += 1;
            if self.row_cursor >= self.row_scroll + visible_rows {
                self.row_scroll = self.row_cursor - visible_rows + 1;
            }
        }
    }

    pub fn cursor_left(&mut self) {
        if self.col_cursor > 0 {
            self.col_cursor -= 1;
            if self.col_cursor < self.col_scroll {
                self.col_scroll = self.col_cursor;
            }
        }
    }

    pub fn cursor_right(&mut self, visible_cols: usize) {
        if self.col_cursor + 1 < self.num_cols() {
            self.col_cursor += 1;
            if self.col_cursor >= self.col_scroll + visible_cols {
                self.col_scroll = self.col_cursor - visible_cols + 1;
            }
        }
    }

    pub fn page_up(&mut self, page: usize) {
        self.row_cursor = self.row_cursor.saturating_sub(page);
        if self.row_cursor < self.row_scroll {
            self.row_scroll = self.row_cursor;
        }
    }

    pub fn page_down(&mut self, page: usize) {
        self.row_cursor = (self.row_cursor + page).min(self.num_rows().saturating_sub(1));
    }

    pub fn home(&mut self) {
        self.row_cursor = 0;
        self.row_scroll = 0;
        self.col_cursor = 0;
        self.col_scroll = 0;
    }

    pub fn end(&mut self) {
        self.row_cursor = self.num_rows().saturating_sub(1);
    }
}

fn parse_csv_row(line: &str) -> Vec<String> {
    let mut fields: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    for ch in line.chars() {
        match ch {
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => {
                fields.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    fields.push(current.trim().to_string());
    fields
}
