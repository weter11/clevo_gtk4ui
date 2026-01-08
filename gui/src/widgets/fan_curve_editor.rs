use egui::{Ui, RichText, Color32};
use egui_plot::{Plot, PlotPoints, Line, Points, Legend, Polygon, PlotPoint};
use tuxedo_common::types::FanCurve;

pub struct FanCurveEditor {
    pub fan_id: u32,
    pub curve: FanCurve,
    selected_point: Option<usize>,
}

impl FanCurveEditor {
    pub fn new(fan_id: u32, curve: FanCurve) -> Self {
        Self {
            fan_id,
            curve,
            selected_point: None,
        }
    }
    
    pub fn show(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.heading(format!("Fan {} Curve", self.fan_id));
            ui.add_space(8.0);
            
            // Graph
            self.draw_graph(ui);
            
            ui.add_space(12.0);
            
            // Points editor
            self.draw_points_editor(ui);
            
            ui.add_space(12.0);
            
            // Controls
            ui.horizontal(|ui| {
                if ui.button("➕ Add Point").clicked() {
                    self.add_point();
                }
                
                if ui.button("🗑️ Remove Selected").clicked() && self.selected_point.is_some() {
                    if self.curve.points.len() > 2 {
                        if let Some(idx) = self.selected_point {
                            self.curve.points.remove(idx);
                            self.selected_point = None;
                        }
                    }
                }
                
                if ui.button("↺ Reset to Default").clicked() {
                    self.reset_to_default();
                }
            });
        });
    }
    
    fn draw_graph(&self, ui: &mut Ui) {
        Plot::new(format!("fan_curve_{}", self.fan_id))
            .height(250.0)
            .width(ui.available_width())
            .show_axes(true)
            .show_grid(true)
            .x_axis_label("Temperature (°C)")
            .y_axis_label("Fan Speed (%)")
            .allow_zoom(false)
            .allow_drag(false)
            .legend(Legend::default())
            .show(ui, |plot_ui| {
                // Draw reference zones first
                self.draw_reference_zones(plot_ui);
                
                // Sort points by temperature
                let mut sorted = self.curve.points.clone();
                sorted.sort_by_key(|p| p.0);
                
                // Draw line
                let line_points: PlotPoints = sorted
                    .iter()
                    .map(|(temp, speed)| [*temp as f64, *speed as f64])
                    .collect();
                
                plot_ui.line(
                    Line::new(line_points)
                        .color(Color32::from_rgb(65, 120, 200))
                        .width(2.0)
                        .name("Fan Curve")
                );
                
                // Draw points
                let points: PlotPoints = sorted
                    .iter()
                    .map(|(temp, speed)| [*temp as f64, *speed as f64])
                    .collect();
                
                plot_ui.points(
                    Points::new(points)
                        .color(Color32::from_rgb(255, 100, 100))
                        .radius(6.0)
                        .name("Control Points")
                );
            });
    }
    
    fn draw_reference_zones(&self, plot_ui: &mut egui_plot::PlotUi) {
        use egui::Stroke;
        
        // Cool zone (0-50°C) - blue tint
        let cool_zone = vec![
            PlotPoint::new(0.0, 0.0),
            PlotPoint::new(50.0, 0.0),
            PlotPoint::new(50.0, 100.0),
            PlotPoint::new(0.0, 100.0),
        ];
        plot_ui.polygon(
            Polygon::new(PlotPoints::Owned(cool_zone))
                .fill_color(Color32::from_rgba_unmultiplied(100, 150, 255, 20))
                .stroke(Stroke::NONE)
        );
        
        // Warm zone (50-70°C) - green tint
        let warm_zone = vec![
            PlotPoint::new(50.0, 0.0),
            PlotPoint::new(70.0, 0.0),
            PlotPoint::new(70.0, 100.0),
            PlotPoint::new(50.0, 100.0),
        ];
        plot_ui.polygon(
            Polygon::new(PlotPoints::Owned(warm_zone))
                .fill_color(Color32::from_rgba_unmultiplied(100, 255, 100, 20))
                .stroke(Stroke::NONE)
        );
        
        // Hot zone (70-85°C) - yellow tint
        let hot_zone = vec![
            PlotPoint::new(70.0, 0.0),
            PlotPoint::new(85.0, 0.0),
            PlotPoint::new(85.0, 100.0),
            PlotPoint::new(70.0, 100.0),
        ];
        plot_ui.polygon(
            Polygon::new(PlotPoints::Owned(hot_zone))
                .fill_color(Color32::from_rgba_unmultiplied(255, 255, 100, 20))
                .stroke(Stroke::NONE)
        );
        
        // Critical zone (85-100°C) - red tint
        let critical_zone = vec![
            PlotPoint::new(85.0, 0.0),
            PlotPoint::new(100.0, 0.0),
            PlotPoint::new(100.0, 100.0),
            PlotPoint::new(85.0, 100.0),
        ];
        plot_ui.polygon(
            Polygon::new(PlotPoints::Owned(critical_zone))
                .fill_color(Color32::from_rgba_unmultiplied(255, 100, 100, 20))
                .stroke(Stroke::NONE)
        );
    }
    
    fn draw_points_editor(&mut self, ui: &mut Ui) {
        ui.label(RichText::new("Control Points:").strong());
        
        // FIXED: Collect changes first, then apply
        let mut changes = Vec::new();
        let mut to_remove = None;
        
        egui::Grid::new(format!("points_grid_{}", self.fan_id))
            .num_columns(4)
            .spacing([12.0, 6.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label(RichText::new("").strong());
                ui.label(RichText::new("Temp (°C)").strong());
                ui.label(RichText::new("Speed (%)").strong());
                ui.label(RichText::new("Actions").strong());
                ui.end_row();
                
                for (idx, (temp, speed)) in self.curve.points.iter().enumerate() {
                    let is_selected = self.selected_point == Some(idx);
                    
                    // Selection indicator
                    if ui.selectable_label(is_selected, format!("{}", idx + 1)).clicked() {
                        self.selected_point = Some(idx);
                    }
                    
                    // Temperature slider
                    let mut temp_val = *temp as f32;
                    if ui.add(egui::Slider::new(&mut temp_val, 0.0..=100.0)
                        .suffix("°C"))
                        .changed() 
                    {
                        changes.push((idx, temp_val as u8, *speed));
                    }
                    
                    // Speed slider
                    let mut speed_val = *speed as f32;
                    if ui.add(egui::Slider::new(&mut speed_val, 0.0..=100.0)
                        .suffix("%"))
                        .changed() 
                    {
                        // Only update if not already in changes
                        if !changes.iter().any(|(i, _, _)| *i == idx) {
                            changes.push((idx, *temp, speed_val as u8));
                        } else {
                            // Update existing change
                            if let Some(change) = changes.iter_mut().find(|(i, _, _)| *i == idx) {
                                change.2 = speed_val as u8;
                            }
                        }
                    }
                    
                    // Delete button
                    if self.curve.points.len() > 2 {
                        if ui.small_button("🗑️").clicked() {
                            to_remove = Some(idx);
                        }
                    } else {
                        ui.label("");
                    }
                    
                    ui.end_row();
                }
            });
        
        // Apply changes outside the iteration
        for (idx, temp, speed) in changes {
            self.curve.points[idx] = (temp, speed);
        }
        
        // Handle removal
        if let Some(idx) = to_remove {
            self.curve.points.remove(idx);
            if self.selected_point == Some(idx) {
                self.selected_point = None;
            }
        }
        
        ui.add_space(6.0);
        ui.label(RichText::new(format!("Total points: {} (min: 2, max: 16)", self.curve.points.len()))
            .small()
            .italics());
    }
    
    fn add_point(&mut self) {
        if self.curve.points.len() >= 16 {
            return;
        }
        
        // Find gap in temperature range
        let mut sorted = self.curve.points.clone();
        sorted.sort_by_key(|p| p.0);
        
        if sorted.is_empty() {
            self.curve.points.push((50, 50));
            return;
        }
        
        // Find largest gap
        let mut best_gap_temp = 50u8;
        let mut best_gap_size = 0u8;
        
        for i in 0..sorted.len().saturating_sub(1) {
            let gap = sorted[i + 1].0.saturating_sub(sorted[i].0);
            if gap > best_gap_size {
                best_gap_size = gap;
                best_gap_temp = sorted[i].0 + gap / 2;
            }
        }
        
        // Check gaps at start and end
        if sorted[0].0 > best_gap_size {
            best_gap_temp = sorted[0].0 / 2;
        }
        
        if let Some(last) = sorted.last() {
            if 100 - last.0 > best_gap_size {
                best_gap_temp = last.0 + (100 - last.0) / 2;
            }
        }
        
        // Interpolate speed
        let speed = self.interpolate_speed(best_gap_temp);
        self.curve.points.push((best_gap_temp, speed));
    }
    
    fn interpolate_speed(&self, temp: u8) -> u8 {
        let mut sorted = self.curve.points.clone();
        sorted.sort_by_key(|p| p.0);
        
        if sorted.is_empty() {
            return 50;
        }
        
        if sorted.len() == 1 {
            return sorted[0].1;
        }
        
        if temp <= sorted[0].0 {
            return sorted[0].1;
        }
        
        if let Some(last) = sorted.last() {
            if temp >= last.0 {
                return last.1;
            }
        }
        
        for i in 0..sorted.len().saturating_sub(1) {
            let (temp1, speed1) = sorted[i];
            let (temp2, speed2) = sorted[i + 1];
            
            if temp >= temp1 && temp <= temp2 {
                let ratio = (temp - temp1) as f32 / (temp2 - temp1) as f32;
                return (speed1 as f32 + ratio * (speed2 as f32 - speed1 as f32)) as u8;
            }
        }
        
        50
    }
    
    fn reset_to_default(&mut self) {
        self.curve.points = vec![
            (0, 0),
            (50, 50),
            (70, 75),
            (85, 100),
        ];
        self.selected_point = None;
    }
    
    pub fn get_curve(&self) -> FanCurve {
        self.curve.clone()
    }
}
