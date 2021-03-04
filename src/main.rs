use rusqlite::types::FromSql;
use druid::widget::{Button, Flex, Label, Split, TextBox, ViewSwitcher};
use druid::{AppLauncher, Data, Env, Lens, LocalizedString, Widget, WidgetExt, WindowDesc};
// use error_chain::error_chain;
use rusqlite::*;
use anyhow::Result;
use regex::Regex;

// error_chain!{
//     foreign_links {
//         SQL(rusqlite::Error)
//     }
// }

#[derive(Clone, Data, Lens)]
struct AppState {
    current_view: u32,
    current_text: String,
}

#[derive(Debug, Clone)]
struct Amount {
    value: u32,
    measurement: String
}

impl Amount {
    
    fn from(value: String) -> Amount {
        let re = Regex::new(r"^(\d+) (lb|k?g|(?:m|d)l|oz)$").unwrap();
        let cap = re.captures(&value).unwrap();
        // let
        Amount { 
            value: cap.get(1).map_or(0, |m| u32::from_str_radix(m.as_str(), 10).unwrap()), 
            measurement: cap.get(2).map_or("UNDEFINED".to_string(), |m| m.as_str().to_string())
        } 
    }
}

// impl FromSql for Amount {
//     fn column_result(value: rusqlite::types::ValueRef<'_>) -> std::result::Result<Self, rusqlite::types::FromSqlError> { 
//         // todo!()
//         // value
//         //     .as_str()
//         //     .and_then(|s| );//.split(' ').collect();
        
//         Ok(Amount { value: 0, measurement: "g".to_string() } )
//     }
// }
// name	tags	amount	cost	calories	carbs	fat	protein
#[derive(Debug, Clone)]
struct Ingredient<'a> {
    name: String,
    tags: Vec<&'a str>,
    amount: Amount,
    cost: f64,
    calories: u32,
    carbs: f64,
    fat: f64,
    protein: f64
}

impl Ingredient<'_> {
    fn describe(self) {
        println!("{} {} of {} costs ${} and contains; {} kcal; {}g carbs; {}g fats; {}g protein.",
            self.amount.value,
            self.amount.measurement,
            self.name,
            self.cost,
            self.calories,
            self.carbs,
            self.fat,
            self.protein
        )
    }
}

fn get_ingredients<'a>() -> Result<Vec<Ingredient<'a>>> {
    let conn = Connection::open("default.db")?;
    let mut stmt = conn.prepare("SELECT * FROM ingredients")?;
    let ingredient_iter = stmt.query_map(params![], |row| {
        // let raw_amount: Amount  = row.get(2)?;

        Ok(Ingredient {
            name: row.get(0)?,
            tags: Vec::from(["hello"]), //(row.get::<usize, String>(1)?).split(',').collect(),
            amount: Amount::from(row.get(2)?),
            cost: row.get(3)?,
            calories: row.get(4)?,
            carbs: row.get(5)?,
            fat: row.get(6)?,
            protein: row.get(7)?,
        })
    })?;

    let mut to_return = Vec::new();
    for ingredient in ingredient_iter {
        to_return.push(ingredient.unwrap().clone());
    }

    Ok(to_return)
}

pub fn main() {
    let ingredients = get_ingredients().unwrap();

    for ingredient in ingredients {
        ingredient.describe();
    }

    let main_window = WindowDesc::new(make_ui).title(LocalizedString::new("Sous Chef"));
    let data = AppState {
        current_view: 0,
        current_text: "Edit me!".to_string(),
    };
    // AppLauncher::with_window(main_window)
    //     .use_simple_logger()
    //     .launch(data)
    //     .expect("launch failed");
}

fn make_ui() -> impl Widget<AppState> {
    let mut switcher_column = Flex::row();
    switcher_column.add_child(
        Label::new(|data: &u32, _env: &Env| format!("Current view: {}", data))
            .lens(AppState::current_view),
    );
    for i in 0..6 {
        switcher_column.add_spacer(80.);
        switcher_column.add_child(
            Button::new(format!("View {}", i))
                .on_click(move |_event, data: &mut u32, _env| {
                    *data = i;
                })
                .lens(AppState::current_view),
        );
    }

    let view_switcher = ViewSwitcher::new(
        |data: &AppState, _env| data.current_view,
        |selector, _data, _env| match selector {
            0 => Box::new(Label::new("Simple Label").center()),
            1 => Box::new(
                Button::new("Simple Button").on_click(|_event, _data, _env| {
                    println!("Simple button clicked!");
                }),
            ),
            2 => Box::new(
                Button::new("Another Simple Button").on_click(|_event, _data, _env| {
                    println!("Another simple button clicked!");
                }),
            ),
            3 => Box::new(
                Flex::column()
                    .with_flex_child(Label::new("Here is a label").center(), 1.0)
                    .with_flex_child(
                        Button::new("Button").on_click(|_event, _data, _env| {
                            println!("Complex button clicked!");
                        }),
                        1.0,
                    )
                    .with_flex_child(TextBox::new().lens(AppState::current_text), 1.0)
                    .with_flex_child(
                        Label::new(|data: &String, _env: &Env| format!("Value entered: {}", data))
                            .lens(AppState::current_text),
                        1.0,
                    ),
            ),
            4 => Box::new(
                Split::columns(
                    Label::new("Left split").center(),
                    Label::new("Right split").center(),
                )
                .draggable(true),
            ),
            _ => Box::new(Label::new("Unknown").center()),
        },
    );

    Flex::column()
        .with_child(switcher_column)
        .with_flex_child(view_switcher, 1.0)
}