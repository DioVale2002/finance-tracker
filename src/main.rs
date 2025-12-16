use eframe::egui;
use egui::{Color32, Pos2, Sense, Stroke, Vec2, Shape};
use egui_plot::{Legend, Line, Plot, PlotPoints, Points}; 
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use chrono::{NaiveDateTime, DateTime, NaiveDate, Local}; 
use std::f64::consts::TAU;

// 1. Data Structures with Serialization
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Default, Debug)]
enum TransactionType {
    Income,
    #[default]
    Expense,
}

// Category Enum
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
enum Category {
    // Income Categories
    Salary,
    Business,
    Investments,
    Gifts,
    
    // Expense Categories
    Food,
    Housing, 
    Transport,
    Utilities,
    Entertainment,
    Shopping,
    Health,
    Education,
    
    // Universal
    Other,
}

impl Default for Category {
    fn default() -> Self {
        Self::Other
    }
}

impl Category {
    fn color(&self) -> Color32 {
        match self {
            Category::Salary => Color32::from_rgb(100, 200, 100),
            Category::Business => Color32::from_rgb(100, 255, 100),
            Category::Investments => Color32::from_rgb(50, 150, 50),
            Category::Gifts => Color32::from_rgb(150, 255, 150),
            
            Category::Food => Color32::from_rgb(255, 100, 100),
            Category::Housing => Color32::from_rgb(200, 50, 50),
            Category::Transport => Color32::from_rgb(100, 100, 255),
            Category::Utilities => Color32::from_rgb(100, 200, 255),
            Category::Entertainment => Color32::from_rgb(255, 165, 0),
            Category::Shopping => Color32::from_rgb(255, 105, 180),
            Category::Health => Color32::from_rgb(255, 50, 50),
            Category::Education => Color32::from_rgb(150, 100, 255),
            
            Category::Other => Color32::GRAY,
        }
    }

    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
    
    fn variants_for_type(t: TransactionType) -> Vec<Category> {
        match t {
            TransactionType::Income => vec![
                Category::Salary, Category::Business, Category::Investments, 
                Category::Gifts, Category::Other
            ],
            TransactionType::Expense => vec![
                Category::Food, Category::Housing, Category::Transport, 
                Category::Utilities, Category::Entertainment, Category::Shopping, 
                Category::Health, Category::Education, Category::Other
            ],
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct Transaction {
    description: String,
    amount: f64,
    trans_type: TransactionType,
    #[serde(default)]
    category: Category,
    date: NaiveDateTime,
}

// 2. Application State
#[derive(Serialize, Deserialize)]
struct FinanceApp {
    transactions: Vec<Transaction>,
    
    #[serde(skip)]
    input_date: NaiveDate, 
    #[serde(skip)]
    input_desc: String,
    #[serde(skip)]
    input_amount: String,
    #[serde(skip)]
    input_type: TransactionType,
    #[serde(skip)]
    input_category: Category,
    #[serde(skip)]
    current_tab: Tab,
    #[serde(skip)]
    editing_index: Option<usize>, // NEW: Tracks which item we are editing
}

#[derive(PartialEq, Default)]
enum Tab {
    #[default]
    Transactions,
    Graph,
}

impl Default for FinanceApp {
    fn default() -> Self {
        Self {
            transactions: Vec::new(),
            input_date: Local::now().date_naive(), 
            input_desc: String::new(),
            input_amount: String::new(),
            input_type: TransactionType::Expense,
            input_category: Category::Food,
            current_tab: Tab::Transactions,
            editing_index: None,
        }
    }
}

impl FinanceApp {
    fn save_data(&self) {
        if let Ok(file) = File::create("finance_data.json") {
            let writer = BufWriter::new(file);
            let _ = serde_json::to_writer(writer, &self);
        }
    }

    fn load_data() -> Self {
        if let Ok(file) = File::open("finance_data.json") {
            let reader = BufReader::new(file);
            if let Ok(app) = serde_json::from_reader(reader) {
                // Return loaded app but reset input fields and editing state
                return FinanceApp {
                    input_date: Local::now().date_naive(),
                    input_desc: String::new(),
                    input_amount: String::new(),
                    input_type: TransactionType::Expense,
                    input_category: Category::Food,
                    current_tab: Tab::Transactions,
                    editing_index: None,
                    ..app
                };
            }
        }
        Self::default()
    }
}

impl eframe::App for FinanceApp {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_data();
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.current_tab, Tab::Transactions, "📝 Transactions");
                ui.selectable_value(&mut self.current_tab, Tab::Graph, "📈 Analytics");
            });
            ui.separator();

            match self.current_tab {
                Tab::Transactions => self.show_transactions_ui(ui),
                Tab::Graph => self.show_analytics_ui(ui),
            }
        });
    }
}

impl FinanceApp {
    fn show_transactions_ui(&mut self, ui: &mut egui::Ui) {
        // Change header based on mode
        if self.editing_index.is_some() {
            ui.heading("Edit Transaction");
        } else {
            ui.heading("Add New Transaction");
        }
        
        ui.horizontal(|ui| {
            ui.label("Date:");
            ui.add(egui_extras::DatePickerButton::new(&mut self.input_date));
            
            ui.add_space(10.0);
            
            ui.label("Desc:");
            ui.text_edit_singleline(&mut self.input_desc);
            ui.label("Amount:");
            ui.text_edit_singleline(&mut self.input_amount);
        });

        ui.horizontal(|ui| {
            if ui.radio_value(&mut self.input_type, TransactionType::Income, "Income").clicked() {
                 self.input_category = Category::Salary;
            }
            if ui.radio_value(&mut self.input_type, TransactionType::Expense, "Expense").clicked() {
                 self.input_category = Category::Food;
            }

            ui.add_space(20.0);
            ui.label("Category:");
            
            egui::ComboBox::from_id_salt("cat_dropdown")
                .selected_text(self.input_category.to_string())
                .show_ui(ui, |ui| {
                    for cat in Category::variants_for_type(self.input_type) {
                        ui.selectable_value(&mut self.input_category, cat, cat.to_string());
                    }
                });

            ui.add_space(20.0);
            
            // Dynamic Button Text (Add vs Update)
            let btn_text = if self.editing_index.is_some() { "Update" } else { "Add" };

            if ui.button(btn_text).clicked() {
                if let Ok(amount) = self.input_amount.trim().parse::<f64>() {
                    if !self.input_desc.is_empty() {
                        
                        // Handle Time Logic
                        let time_part = if let Some(idx) = self.editing_index {
                            // If editing, preserve the original time of the transaction
                            self.transactions[idx].date.time()
                        } else {
                            // If adding new, use current time
                            Local::now().time()
                        };
                        let full_date_time = self.input_date.and_time(time_part);

                        let new_trans = Transaction {
                            description: self.input_desc.clone(),
                            amount,
                            trans_type: self.input_type,
                            category: self.input_category,
                            date: full_date_time,
                        };

                        if let Some(idx) = self.editing_index {
                            // UPDATE existing
                            self.transactions[idx] = new_trans;
                            self.editing_index = None; // Exit edit mode
                        } else {
                            // ADD new
                            self.transactions.push(new_trans);
                        }

                        // Clear inputs
                        self.input_desc.clear();
                        self.input_amount.clear();
                        // Reset defaults for next add
                        self.input_date = Local::now().date_naive();
                        self.save_data();
                    }
                }
            }

            // Cancel Button (only visible when editing)
            if self.editing_index.is_some() {
                if ui.button("Cancel").clicked() {
                    self.editing_index = None;
                    self.input_desc.clear();
                    self.input_amount.clear();
                    self.input_date = Local::now().date_naive();
                }
            }
        });
        ui.separator();

        let total_balance: f64 = self.transactions.iter().map(|t| {
            match t.trans_type {
                TransactionType::Income => t.amount,
                TransactionType::Expense => -t.amount,
            }
        }).sum();

        ui.heading(format!("Balance: ${:.2}", total_balance));
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut to_remove = None;
            let mut to_edit = None;

            for (index, t) in self.transactions.iter().enumerate().rev() {
                ui.horizontal(|ui| {
                    ui.label(t.date.format("%Y-%m-%d %H:%M").to_string());
                    
                    let (symbol, color) = match t.trans_type {
                        TransactionType::Income => ("+", egui::Color32::GREEN),
                        TransactionType::Expense => ("-", egui::Color32::RED),
                    };
                    
                    ui.colored_label(t.category.color(), format!("[{}]", t.category.to_string()));
                    ui.colored_label(color, symbol);
                    ui.label(format!("${:.2} - {}", t.amount, t.description));
                    
                    // Edit Button (Pencil)
                    if ui.button("✏").clicked() {
                        to_edit = Some(index);
                    }

                    // Delete Button
                    if ui.button("🗑").clicked() {
                        to_remove = Some(index);
                    }
                });
            }

            // Handle Actions
            if let Some(index) = to_edit {
                self.editing_index = Some(index);
                // Populate fields with data from the transaction we want to edit
                let t = &self.transactions[index];
                self.input_desc = t.description.clone();
                self.input_amount = t.amount.to_string();
                self.input_type = t.trans_type;
                self.input_category = t.category;
                self.input_date = t.date.date();
            }

            if let Some(index) = to_remove {
                // If we delete the item being edited, exit edit mode
                if self.editing_index == Some(index) {
                    self.editing_index = None;
                    self.input_desc.clear();
                    self.input_amount.clear();
                } else if let Some(edit_idx) = self.editing_index {
                    // Adjust index if we delete something before the item being edited (rare in reverse loop but good practice)
                    if index < edit_idx {
                        self.editing_index = Some(edit_idx - 1);
                    }
                }
                
                self.transactions.remove(index);
                self.save_data();
            }
        });
    }

    fn show_analytics_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Balance History");
        let available_height = ui.available_height();
        let plot_height = available_height * 0.5;
        
        ui.push_id("line_graph", |ui| {
            let mut sorted_trans = self.transactions.clone();
            sorted_trans.sort_by_key(|t| t.date);

            let mut running_balance = 0.0;
            let mut points: Vec<[f64; 2]> = Vec::new();
            let mut tooltips: Vec<(f64, f64, String, f64, TransactionType)> = Vec::new();

            for t in sorted_trans {
                match t.trans_type {
                    TransactionType::Income => running_balance += t.amount,
                    TransactionType::Expense => running_balance -= t.amount,
                }
                let x = t.date.and_utc().timestamp() as f64; 
                points.push([x, running_balance]);
                tooltips.push((x, running_balance, t.description.clone(), t.amount, t.trans_type));
            }

            if points.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    ui.label("No transactions yet. Add some data to see the graph!");
                    ui.add_space(20.0);
                });
            } else {
                Plot::new("balance_plot")
                    .height(plot_height)
                    .allow_zoom(true)
                    .allow_drag(true)
                    .legend(Legend::default())
                    .auto_bounds(egui::Vec2b::TRUE)
                    .x_axis_formatter(|x, _range| {
                        let val = x.value; 
                        if let Some(dt) = DateTime::from_timestamp(val as i64, 0) {
                            dt.naive_utc().format("%Y-%m-%d\n%H:%M").to_string()
                        } else {
                            String::new()
                        }
                    })
                    .label_formatter(move |name, value| {
                         if name != "Balance" { return String::new(); }
                         
                         let closest = tooltips.iter().min_by(|a, b| {
                             let dist_a = (a.0 - value.x).abs();
                             let dist_b = (b.0 - value.x).abs();
                             dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
                         });
                         
                         if let Some((x, y, desc, amt, t_type)) = closest {
                             if (x - value.x).abs() < 86400.0 { 
                                 let date_str = DateTime::from_timestamp(*x as i64, 0)
                                     .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                                     .unwrap_or_default();
                                 
                                 let (sign, color_name) = match t_type {
                                     TransactionType::Income => ("+", "Income"),
                                     TransactionType::Expense => ("-", "Expense"),
                                 };

                                 return format!(
                                     "Date: {}\nTransaction: {}\nAmount: {}${:.2} ({})\nBalance: ${:.2}", 
                                     date_str, desc, sign, amt, color_name, y
                                 );
                             }
                         }
                         format!("Balance: ${:.2}", value.y)
                    })
                    .show(ui, |plot_ui| {
                        plot_ui.line(Line::new(PlotPoints::from(points.clone())).name("Balance").width(2.0).color(egui::Color32::LIGHT_BLUE));
                        plot_ui.points(Points::new(PlotPoints::from(points)).radius(4.0).color(egui::Color32::LIGHT_BLUE));
                    });
            }
        });

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(20.0);

        ui.heading("Expense Breakdown");
        
        let mut category_totals: std::collections::HashMap<Category, f64> = std::collections::HashMap::new();
        let mut total_expenses = 0.0;
        
        for t in &self.transactions {
            if t.trans_type == TransactionType::Expense {
                *category_totals.entry(t.category).or_insert(0.0) += t.amount;
                total_expenses += t.amount;
            }
        }

        if total_expenses > 0.0 {
            ui.horizontal(|ui| {
                self.draw_pie_chart(ui, &category_totals, total_expenses);
                ui.add_space(40.0);

                ui.vertical(|ui| {
                    let mut sorted_cats: Vec<_> = category_totals.iter().collect();
                    sorted_cats.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

                    for (cat, amount) in sorted_cats {
                        let percentage = (amount / total_expenses) * 100.0;
                        ui.horizontal(|ui| {
                            let (rect, _resp) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::hover());
                            ui.painter().rect_filled(rect, 3.0, cat.color());
                            
                            ui.label(format!("{} ({:.1}%)", cat.to_string(), percentage));
                            ui.label(format!("${:.2}", amount));
                        });
                    }
                });
            });
        } else {
            ui.label("No expenses to show.");
        }
    }

    fn draw_pie_chart(&self, ui: &mut egui::Ui, data: &std::collections::HashMap<Category, f64>, total: f64) {
        let size = 200.0;
        let (rect, _response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
        
        let center = rect.center();
        let radius = size / 2.0;
        
        let mut sorted_data: Vec<_> = data.iter().collect();
        sorted_data.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));

        let mut current_angle = -TAU / 4.0;

        for (cat, amount) in sorted_data {
            let slice_angle = (amount / total) * TAU;
            let color = cat.color();

            let points_on_arc = 30;
            let mut points = vec![center];

            for i in 0..=points_on_arc {
                let t = i as f64 / points_on_arc as f64;
                let angle = current_angle + t * slice_angle;
                let x = center.x + radius * angle.cos() as f32;
                let y = center.y + radius * angle.sin() as f32;
                points.push(Pos2::new(x, y));
            }

            ui.painter().add(Shape::convex_polygon(points, color, Stroke::new(1.0, Color32::BLACK)));

            current_angle += slice_angle;
        }
    }
}

fn main() -> eframe::Result<()> {
    // FORCE WSL COMPATIBILITY (The "Nuclear Option")
    std::env::set_var("WINIT_UNIX_BACKEND", "x11");
    std::env::set_var("LIBGL_ALWAYS_SOFTWARE", "1");

    println!("Starting Finance Tracker in WSL Compatibility Mode (X11 + Software Rendering)...");

    let app = FinanceApp::load_data();
    
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_transparent(false) 
            .with_icon(eframe::icon_data::from_png_bytes(&[]).unwrap_or_default()), 
        vsync: false, 
        multisampling: 0, 
        depth_buffer: 0,
        stencil_buffer: 0,
        ..Default::default()
    };
    
    eframe::run_native(
        "Rust Finance Tracker v6", // Bumped version
        native_options,
        Box::new(|_cc| Ok(Box::new(app))),
    )
}