use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use serde::Serialize;
use zip::ZipArchive;

use crate::food::{self, Food};
use roxmltree::{Document, Node};

pub fn read_meal_docx<P: AsRef<Path>>(path: P) -> Vec<MealRow> {
    let docx_file = File::open(path).expect("A bemeneti fajl nem talalhato.");
    let docx_reader = BufReader::new(docx_file);
    let mut zip = ZipArchive::new(docx_reader)
        .expect("A bemeneti fajl nem megfelelo formatumu (1)");

    let mut zipped_document = zip
        .by_name("word/document.xml")
        .expect("A bemeneti fajl nem megfelelo formatumu (2)");

    let mut xml = String::new();
    zipped_document
        .read_to_string(&mut xml)
        .expect("Nem sikerult a fajlbeolvasas");

    #[rustfmt::skip]
    let xml = Document::parse(&xml)
        .expect("A bemeneti fajl nem megfelelo formatumu (3)");

    xml.descendants()
        .filter(|e| e.has_tag_name("tbl")) // Find tables
        .flat_map(|tbl| read_row_data(&tbl)) // Extract meal data
        .collect()
}

#[derive(Debug, Serialize)]
pub struct MealRow {
    header: String,
    cells: Vec<MealCell>,
}

#[derive(Debug, Serialize)]
pub struct MealCell {
    food: Vec<Food>,
    nutrients: Option<Nutrients>,
}

#[derive(Debug, Serialize)]
pub struct Nutrients {
    energy: f64,
    carbs: f64,
    protein: f64,
    sugar: f64,
    fat: f64,
    salt: f64,
    saturated_fat: f64,
}

impl Nutrients {
    pub fn from_vec(v: &Vec<f64>) -> Nutrients {
        Nutrients {
            energy: v[0],
            carbs: v[1],
            protein: v[2],
            sugar: v[3],
            fat: v[4],
            salt: v[5],
            saturated_fat: v[6],
        }
    }
}

impl MealCell {
    /// Interpret the extracted (string) cell values as [Food] and [Nutrients]
    /// objects and combine them into a [MealCell].
    pub fn from_cell_values(
        food_cell_value: &str,
        nutrient_cell_value: &str,
    ) -> MealCell {
        let food = food::parse(food_cell_value);
        let nutrients = match parse_numbers(nutrient_cell_value) {
            v if v.len() >= 6 => Some(Nutrients::from_vec(&v)),
            _ => None,
        };
        MealCell { food, nutrients }
    }
}

/// Read all rows of a menu and convert them into strongly typed [MealRow]s.
fn read_row_data<'a>(table: &roxmltree::Node<'a, 'a>) -> Vec<MealRow> {
    let (even, odd): (Vec<_>, Vec<_>) = table
        .children()
        // Find row elements
        .filter(|c| c.has_tag_name("tr"))
        // Ignore header
        .skip(1)
        .enumerate()
        // Separate odd (food) and even (nutrition information) rows
        .partition(|(idx, _)| idx % 2 == 0);

    // Extract text from `<wd:t>` elements for each cell
    let food_rows = even.iter().map(|(_, row)| read_cell_text(row));
    let nutrient_rows = odd.iter().map(|(_, row)| read_cell_text(row));

    // Join the two rows into a single `MealRow`
    let parsed_row = food_rows
        .zip(nutrient_rows)
        .map(|((header, food_cells), (_, nutrient_cells))| {
            let cells = food_cells
                .iter()
                .zip(nutrient_cells.iter())
                .map(|(food, nutr)| MealCell::from_cell_values(food, nutr))
                .collect::<Vec<_>>();

            MealRow { header, cells }
        })
        .collect::<Vec<_>>();

    parsed_row
}

/// Read the text contents of each cell in a `<w:tr>`. Results are concatenated
/// using newlines `\n` and collected into a [String].
fn read_cell_text<'a>(row: &roxmltree::Node<'a, 'a>) -> (String, Vec<String>) {
    // Text is stored in `<w:t>` elements. Extract these and join them
    // into a single string.
    let get_text = |c: Node| {
        c.descendants()
            .filter(|content| content.has_tag_name("t"))
            .flat_map(|content| content.text())
            .collect::<Vec<_>>()
            .join("\n")
    };

    let mut cells = row.children().filter(|c| c.has_tag_name("tc"));
    let header = cells
        .next()
        .map_or("?".to_owned(), get_text)
        .replace("\n", "");
    let content = cells.map(get_text).collect::<Vec<_>>();

    (header, content)
}

/// Parse numbers from input. Accepts numbers with either `.` or `,` as the
/// decimal separator.
fn parse_numbers(input: &str) -> Vec<f64> {
    struct State {
        buf: Vec<char>,
        has_separator: bool,
    }

    impl State {
        fn read(&self) -> Option<f64> {
            if self.buf.is_empty() {
                return None;
            }

            let sval: String = self.buf.iter().collect();
            let nval = sval.parse::<f64>();
            nval.ok()
        }

        fn reset(&mut self) {
            self.buf.clear();
            self.has_separator = false;
        }

        fn pushc(&mut self, c: char) {
            self.buf.push(c);
        }

        fn accepts_separator(&self) -> bool {
            !self.buf.is_empty() && !self.has_separator
        }

        fn push_separator(&mut self) {
            self.buf.push('.');
            self.has_separator = true;
        }
    }

    let mut chars = input.chars();
    let mut numbers: Vec<f64> = vec![];
    let mut state = State {
        buf: vec![],
        has_separator: false,
    };

    while let Some(c) = chars.next() {
        match c {
            c @ '0'..='9' => state.pushc(c),
            '.' | ',' if state.accepts_separator() => {
                state.push_separator();
            }
            _ => match state.read() {
                Some(nval) => {
                    numbers.push(nval);
                    state.reset()
                }
                _ => {}
            },
        }
    }

    numbers
}
