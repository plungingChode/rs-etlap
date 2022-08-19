use config::Config;

mod cursor;
mod food;
mod meal_parser;
mod config;

fn main() {
    let cfg = Config::load("config.toml");
    let rows = meal_parser::read_meal_docx(cfg.input_path);
}
