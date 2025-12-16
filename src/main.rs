use eframe::egui;
use egui::{Color32, Pos2, Sense, Stroke, Vec2, Shape};
use egui_plot::{Legend, Line, Plot, PlotPoints}; // Removed Text/PlotPoint as we use simple labels now
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use chrono::{NaiveDateTime, DateTime};
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
    input_desc: String,
    #[serde(skip)]
    input_amount: String,
    #[serde(skip)]
    input_type: TransactionType,
    #[serde(skip)]
    input_category: Category,
    #[serde(skip)]
    current_tab: Tab,
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
            input_desc: String::new(),
            input_amount: String::new(),
            input_type: TransactionType::Expense,
            input_category: Category::Food,
            current_tab: Tab::Transactions,
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
                return app;
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
        ui.heading("Add New Transaction");
        
        ui.horizontal(|ui| {
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
            
            if ui.button("Add").clicked() {
                if let Ok(amount) = self.input_amount.trim().parse::<f64>() {
                    if !self.input_desc.is_empty() {
                        let new_trans = Transaction {
                            description: self.input_desc.clone(),
                            amount,
                            trans_type: self.input_type,
                            category: self.input_category,
                            date: chrono::Local::now().naive_local(),
                        };
                        self.transactions.push(new_trans);
                        self.input_desc.clear();
                        self.input_amount.clear();
                        self.save_data();
                    }
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
                    
                    if ui.button("🗑").clicked() {
                        to_remove = Some(index);
                    }
                });
            }
            if let Some(index) = to_remove {
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

            for t in sorted_trans {
                match t.trans_type {
                    TransactionType::Income => running_balance += t.amount,
                    TransactionType::Expense => running_balance -= t.amount,
                }
                let x = t.date.and_utc().timestamp() as f64; 
                points.push([x, running_balance]);
            }

            if points.is_empty() {
                // FALLBACK: Completely bypass Plot logic if empty to prevent crashes
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
                    .show(ui, |plot_ui| {
                        plot_ui.line(Line::new(PlotPoints::from(points)).name("Balance").width(2.0).color(egui::Color32::LIGHT_BLUE));
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
        
        let mut current_angle = -TAU / 4.0;

        for (cat, amount) in data {
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
    // We hardcode these to ensure the app ALWAYS uses the stable path on WSL.
    
    // 1. Force X11 backend (Since 'xeyes' works for you, this is the safe bet)
    std::env::set_var("WINIT_UNIX_BACKEND", "x11");
    
    // 2. Force Software Rendering (Prevents "Broken pipe" / GPU driver crashes)
    std::env::set_var("LIBGL_ALWAYS_SOFTWARE", "1");

    println!("Starting Finance Tracker in WSL Compatibility Mode (X11 + Software Rendering)...");

    let app = FinanceApp::load_data();
    
    // "Safe Mode" Options for WSL
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_transparent(false) // transparency causes crashes in WSL
            .with_icon(eframe::icon_data::from_png_bytes(&[]).unwrap_or_default()), // Prevent icon crash
        vsync: false, // vsync causes "broken pipe" in WSL often
        multisampling: 0, // disable anti-aliasing to save the software renderer
        depth_buffer: 0,
        stencil_buffer: 0,
        ..Default::default()
    };
    
    // Bumped to v4 to clear any corrupted window state from previous crashes
    eframe::run_native(
        "Rust Finance Tracker v4",
        native_options,
        Box::new(|_cc| Ok(Box::new(app))),
    )
}