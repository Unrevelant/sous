// use rusqlite::types::FromSql;
use druid::widget::{Button, Flex, Label, Split, TextBox, ViewSwitcher, CrossAxisAlignment, Scroll, List};
use druid::{AppLauncher, Data, Env, Lens, LocalizedString, Widget, WidgetExt, WindowDesc};
use druid::lens::{self, LensExt};
use druid::im::Vector;
use druid::{UnitPoint, Color};
// use error_chain::error_chain;
use rusqlite::*;
use anyhow::Result;
use regex::Regex;
// use std::rc::Rc;

// error_chain!{
//     foreign_links {
//         SQL(rusqlite::Error)
//     }
// }

#[derive(Clone, Data, Lens)]
struct AppState {
    current_view: u32,
    current_text: String,
    ingredients: Vector<Ingredient>
}

enum View {
    IngredientList,
    IngredientEdit,
    RecipeList,
    RecipeEdit,
    PantryList,
    PantryEdit,
}

// struct IngredientLens(Vector<Ingredient>);

// impl Lens<AppState, Vector<Ingredient>> for IngredientLens {
//     fn with<V, F: FnOnce(&Vector<Ingredient>) -> V>(&self, data: &AppState, f: F) -> V {
//         let v = data.ingredients.clone();
//         f(&v)
//         // todo!()
//     }

//     fn with_mut<V, F: FnOnce(&mut Vector<Ingredient>) -> V>(&self, data: &mut AppState, f: F) -> V {
//         todo!()
//     }
// }

#[derive(Debug, Clone, Data, Lens)]
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
#[derive(Debug, Clone, Lens)]
struct Ingredient {
    name: String,
    tags: Vec<&'static str>,
    amount: Amount,
    cost: f64,
    calories: u32,
    carbs: f64,
    fat: f64,
    protein: f64
}

impl Data for Ingredient {
    fn same(&self, other: &Self) -> bool { self.name == other.name }
}

// impl Widget<Ingredient> for Ingredient {
//     fn event(&mut self, ctx: &mut druid::EventCtx, event: &druid::Event, data: &mut Ingredient, env: &Env) {
//         todo!()
//     }

//     fn lifecycle(&mut self, ctx: &mut druid::LifeCycleCtx, event: &druid::LifeCycle, data: &Ingredient, env: &Env) {
//         todo!()
//     }

//     fn update(&mut self, ctx: &mut druid::UpdateCtx, old_data: &Ingredient, data: &Ingredient, env: &Env) {
//         todo!()
//     }

//     fn layout(&mut self, ctx: &mut druid::LayoutCtx, bc: &druid::BoxConstraints, data: &Ingredient, env: &Env) -> druid::Size {
//         todo!()
//     }

//     fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &Ingredient, env: &Env) {
//         todo!()
//     }
// }

impl Ingredient {
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

    fn make_box(self) -> Box<dyn Widget<Ingredient>> {
        Box::new(Flex::row().with_flex_child(Flex::column().with_flex_child(Label::new(self.name), 1.0), 1.0))
    }
}

fn get_ingredients<'a>() -> Result<Vec<Ingredient>> {
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

    // for ingredient in ingredients {
    //     ingredient.describe();
    // }

    let main_window = WindowDesc::new(make_ui).title(LocalizedString::new("Sous Chef"));
    let data = AppState {
        current_view: 0,
        current_text: "Edit me!".to_string(),
        ingredients: Vector::from(ingredients)
    };
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(data)
        .expect("launch failed");
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

    //.border(Color::WHITE, 1.0)

    // let ingredient_list = Scroll::new(
    //     List::new(|| {
    //         Label::new(
    //             |ing: &Ingredient, _env: &_| format!("{}", ing.name)
    //         )
    //         .align_vertical(UnitPoint::LEFT)
    //         .padding(10.0)
    //         .expand()
    //         .height(50.0)
    //     }))
    //     .vertical();
    //     .lens(AppState::ingredients);
        
        // {

    let view_switcher = ViewSwitcher::new(
        |data: &AppState, _env| data.current_view,
        |selector, _data, _env| match selector {
            0 => Box::new(
                Scroll::new(
                    List::new(|| {Flex::row().with_child(
                        Flex::column().with_child(
                            Label::new(
                                |ing: &Ingredient, _env: &_| format!("{}", ing.name)
                            )
                            .align_vertical(UnitPoint::LEFT)
                            .padding(10.0)
                            // .expand()
                        ).with_child(
                            Label::new(
                                |ing: &Ingredient, _env: &_| format!("{:?}", ing.tags)
                            )
                            .align_vertical(UnitPoint::LEFT)
                            .padding(10.0)
                            // .expand()
                        )).with_child(
                        Flex::column().with_child(
                            Label::new(
                                |ing: &Ingredient, _env: &_| format!("{}{} costs ${}", ing.amount.value, ing.amount.measurement, ing.cost)
                            )
                            .align_vertical(UnitPoint::LEFT)
                            .padding(10.0)
                            // .expand()
                        ).with_child(
                            Label::new(
                                |ing: &Ingredient, _env: &_| format!("{} calories", ing.calories)
                            )
                            .align_vertical(UnitPoint::LEFT)
                            .padding(10.0)
                            // .expand()
                        )
                        ).must_fill_main_axis(true).border(Color::WHITE, 1.0)
                    }))
                    .vertical()
                    .lens(AppState::ingredients)
            ),
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
        // }

    Flex::column()
        .with_child(switcher_column)
        .with_flex_child(view_switcher, 1.0)
        // .debug_paint_layout()
}