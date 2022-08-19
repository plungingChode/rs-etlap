use std::collections::HashSet;
use std::{fs, io::BufWriter};

use config::Config;
use meal_parser::{Nutrients, MealCell};

mod config;
mod cursor;
mod food;
mod meal_parser;

fn main() {
    let cfg = Config::load("config.toml");

    let ignored_rows = cfg.row_filter.split(",").map(|s| s.to_string());
    let ignored_rows: HashSet<String> = HashSet::from_iter(ignored_rows);

    let rows = meal_parser::read_meal_docx(cfg.input_path);
    let rows = rows.iter().filter(|r| !ignored_rows.contains(&r.header));

    let mut buf = String::new();
    for row in rows {
        let row_header = format!("\"{}\"{}", row.header, cfg.delimiter);
        buf.push_str(&row_header);
        // Food row
        for cell in row.cells.iter() {
            buf.push_str(&cell.food_to_csv());
            buf.push(cfg.delimiter);
        }
        buf.pop(); // remove unnecessary delimiter
        buf.push('\n');

        // Nutrient row
        buf.push_str(&row_header);
        for cell in row.cells.iter() {
            buf.push_str(&cell.nutrients_to_csv());
            buf.push(cfg.delimiter)
        }
        buf.pop(); 
        buf.push('\n');
    }

    fs::write(cfg.output_path, buf).unwrap();
}

impl MealCell {
    fn food_to_csv(&self) -> String {
        let food = self.food
            .iter()
            .map(|f| {
                let allergens = match f.allergens.clone() {
                    Some(a) => a,
                    None => String::new(),
                };

                format!("{} {}", f.name, allergens)
            })
            .collect::<Vec<_>>()
            .join("\n");
        let food = format!("\"{}\"", food);
        food
    }

    fn nutrients_to_csv(&self) -> String {
        if let Some(n) = &self.nutrients {
            // Note the start/end quotes!
            let nutrients = [
                format!("energia: {:0} kcal", n.energy),
                format!("szénhidrát: {:.1}g", n.carbs),
                format!("fehérje: {:.1}g", n.protein),
                format!("zsír: {:.1}g", n.fat),
                format!("curkor: {:.1}g", n.sugar),
                format!("só: {:.1}g", n.salt),
                format!("telített zsírok: {:.1}g", n.saturated_fat),
            ];
            let nutrients = nutrients.join("\n");
            let nutrients = format!("\"{}\"", nutrients);
            nutrients
        } else {
            String::new()
        }
    }
}
