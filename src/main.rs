use eframe::egui;
use egui_plot::{Legend, Line, Plot, PlotPoints};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use chrono::NaiveDateTime; // Changed from NaiveDate to NaiveDateTime

// 1. Data Structures with Serialization (Saving)
#[derive(Clone, PartialEq, Serialize, Deserialize, Default)]
enum TransactionType {
    Income,
    #[default]
    Expense,
}

#[derive(Clone, Serialize, Deserialize)]
struct Transaction {
    description: String,
    amount: f64,
    trans_type: TransactionType,
    date: NaiveDateTime, // FIX: Now stores exact time, not just date
}

// 2. Application State
#[derive(Serialize, Deserialize)]
struct FinanceApp {
    transactions: Vec<Transaction>,
    
    // We skip saving these temporary input fields
    #[serde(skip)]
    input_desc: String,
    #[serde(skip)]
    input_amount: String,
    #[serde(skip)]
    input_type: TransactionType,
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
            current_tab: Tab::Transactions,
        }
    }
}

impl FinanceApp {
    // Helper to save data to a JSON file
    fn save_data(&self) {
        if let Ok(file) = File::create("finance_data.json") {
            let writer = BufWriter::new(file);
            let _ = serde_json::to_writer(writer, &self);
        }
    }

    // Helper to load data from JSON file
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
    // Save on exit
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_data();
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            
            // --- TOP NAVIGATION ---
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.current_tab, Tab::Transactions, "📝 Transactions");
                ui.selectable_value(&mut self.current_tab, Tab::Graph, "📈 Analytics");
            });
            ui.separator();

            match self.current_tab {
                Tab::Transactions => self.show_transactions_ui(ui),
                Tab::Graph => self.show_graph_ui(ui),
            }
        });
    }
}

// UI Implementation Details
impl FinanceApp {
    fn show_transactions_ui(&mut self, ui: &mut egui::Ui) {
        // --- INPUT SECTION ---
        ui.heading("Add New Transaction");
        ui.horizontal(|ui| {
            ui.label("Desc:");
            ui.text_edit_singleline(&mut self.input_desc);
            ui.label("Amount:");
            ui.text_edit_singleline(&mut self.input_amount);
        });

        ui.horizontal(|ui| {
            ui.radio_value(&mut self.input_type, TransactionType::Income, "Income");
            ui.radio_value(&mut self.input_type, TransactionType::Expense, "Expense");
            
            if ui.button("Add").clicked() {
                if let Ok(amount) = self.input_amount.trim().parse::<f64>() {
                    if !self.input_desc.is_empty() {
                        let new_trans = Transaction {
                            description: self.input_desc.clone(),
                            amount,
                            trans_type: self.input_type.clone(),
                            // FIX: Capture exact current time (seconds included)
                            date: chrono::Local::now().naive_local(), 
                        };
                        self.transactions.push(new_trans);
                        self.input_desc.clear();
                        self.input_amount.clear();
                        self.save_data(); // Auto-save on add
                    }
                }
            }
        });
        ui.separator();

        // --- HISTORY LIST ---
        let total_balance: f64 = self.transactions.iter().map(|t| {
            match t.trans_type {
                TransactionType::Income => t.amount,
                TransactionType::Expense => -t.amount,
            }
        }).sum();

        ui.heading(format!("Balance: ${:.2}", total_balance));
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut to_remove = None;
            // Iterate reversed to show newest at top
            for (index, t) in self.transactions.iter().enumerate().rev() {
                ui.horizontal(|ui| {
                    // Display Date AND Time in the list for clarity
                    ui.label(t.date.format("%Y-%m-%d %H:%M").to_string());
                    
                    let (symbol, color) = match t.trans_type {
                        TransactionType::Income => ("+", egui::Color32::GREEN),
                        TransactionType::Expense => ("-", egui::Color32::RED),
                    };
                    ui.colored_label(color, symbol);
                    ui.label(format!("${:.2} - {}", t.amount, t.description));
                    
                    if ui.button("🗑").clicked() {
                        to_remove = Some(index);
                    }
                });
            }
            if let Some(index) = to_remove {
                self.transactions.remove(index);
                self.save_data(); // Auto-save on delete
            }
        });
    }

    fn show_graph_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Balance History");
        
        // 1. Sort transactions by full timestamp
        let mut sorted_trans = self.transactions.clone();
        sorted_trans.sort_by_key(|t| t.date);

        // 2. Calculate running balance
        let mut running_balance = 0.0;
        let mut points: Vec<[f64; 2]> = Vec::new();

        // FIX: Removed the artificial "Zero" start point. 
        // The graph now faithfully starts at the first transaction.

        for t in sorted_trans {
            match t.trans_type {
                TransactionType::Income => running_balance += t.amount,
                TransactionType::Expense => running_balance -= t.amount,
            }
            // FIX: X-Axis is now Timestamp (Seconds) instead of Days
            let x = t.date.timestamp() as f64; 
            points.push([x, running_balance]);
        }

        // 3. Draw the Plot
        Plot::new("my_plot")
            .min_size(ui.available_size()) // Fill remaining space
            .allow_zoom(true)
            .allow_drag(true)
            .auto_bounds(egui::Vec2b::TRUE) 
            .legend(Legend::default())
            // FIX: Formatter now handles seconds timestamp to Date Time string
            .x_axis_formatter(|x, _range| {
                let val = x.value; 
                // Convert timestamp (seconds) back to datetime string
                if let Some(date) = NaiveDateTime::from_timestamp_opt(val as i64, 0) {
                    date.format("%Y-%m-%d\n%H:%M").to_string() // Multiline for better fit
                } else {
                    String::new()
                }
            })
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new(PlotPoints::from(points))
                        .name("Balance")
                        .width(2.0)
                        .color(egui::Color32::LIGHT_BLUE)
                );
            });
    }
}

fn main() -> eframe::Result<()> {
    // Load old data if it exists (Note: Format change might reset data)
    let app = FinanceApp::load_data();
    
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Rust Finance Tracker v2",
        native_options,
        Box::new(|_cc| Ok(Box::new(app))),
    )
}