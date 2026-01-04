use gtk::prelude::*;
use gtk::{Box, Button, DrawingArea, Label, Orientation};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use tuxedo_common::types::FanCurve;

#[derive(Clone)]
pub struct FanCurveEditor {
    pub container: Box,
    pub curve: Rc<RefCell<FanCurve>>,
    drawing_area: DrawingArea,
}

impl FanCurveEditor {
    pub fn new(fan_id: u32, initial_curve: FanCurve) -> Self {
        let container = Box::new(Orientation::Vertical, 12);
        container.set_margin_top(12);
        container.set_margin_bottom(12);
        container.set_margin_start(12);
        container.set_margin_end(12);

        let curve = Rc::new(RefCell::new(initial_curve));

        // Title
        let title = Label::new(Some(&format!("Fan {} Curve Editor", fan_id)));
        title.add_css_class("title-3");
        container.append(&title);

        // Description
        let description = Label::new(Some("Set temperature (°C) and fan speed (%) points"));
        description.add_css_class("dim-label");
        container.append(&description);

        // Drawing area for curve visualization
        let drawing_area = DrawingArea::new();
        drawing_area.set_content_width(400);
        drawing_area.set_content_height(300);
        drawing_area.set_vexpand(true);
        drawing_area.set_hexpand(true);
        
        let curve_clone = curve.clone();
        drawing_area.set_draw_func(move |_da, cr, width, height| {
            Self::draw_curve(cr, width, height, &curve_clone.borrow());
        });

        container.append(&drawing_area);

        // Points editor
        let points_group = adw::PreferencesGroup::builder()
            .title("Curve Points")
            .description("Add points to define the fan curve")
            .build();

        let points_list = Box::new(Orientation::Vertical, 6);
        
        // Display existing points
        for (i, (temp, speed)) in curve.borrow().points.iter().enumerate() {
            let point_row = Self::create_point_row(
                i,
                *temp,
                *speed,
                curve.clone(),
                drawing_area.clone(),
            );
            points_list.append(&point_row);
        }

        let scrolled = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .min_content_height(200)
            .max_content_height(400)
            .child(&points_list)
            .build();

        points_group.add(&scrolled);
        container.append(&points_group);

        // Add point button
        let add_button = Button::with_label("➕ Add Point");
        add_button.add_css_class("suggested-action");
        
        let curve_clone = curve.clone();
        let drawing_clone = drawing_area.clone();
        let points_clone = points_list.clone();
        add_button.connect_clicked(move |_| {
            let mut crv = curve_clone.borrow_mut();
            if crv.points.len() < 16 {
                // Add new point at reasonable defaults
                let new_temp = if crv.points.is_empty() {
                    50
                } else {
                    crv.points.last().unwrap().0 + 10
                };
                let new_speed = 50;
                crv.points.push((new_temp, new_speed));
                
                // Add UI row
                let idx = crv.points.len() - 1;
                drop(crv); // Release borrow
                
                let point_row = Self::create_point_row(
                    idx,
                    new_temp,
                    new_speed,
                    curve_clone.clone(),
                    drawing_clone.clone(),
                );
                points_clone.append(&point_row);
                drawing_clone.queue_draw();
            }
        });

        container.append(&add_button);

        // Info labels
        let info_box = Box::new(Orientation::Vertical, 6);
        info_box.set_margin_top(12);
        
        let info1 = Label::new(Some("• Temperature: 0-100°C"));
        info1.set_halign(gtk::Align::Start);
        info1.add_css_class("caption");
        info_box.append(&info1);
        
        let info2 = Label::new(Some("• Fan Speed: 0-100%"));
        info2.set_halign(gtk::Align::Start);
        info2.add_css_class("caption");
        info_box.append(&info2);
        
        let info3 = Label::new(Some("• At least 2 points required"));
        info3.set_halign(gtk::Align::Start);
        info3.add_css_class("caption");
        info_box.append(&info3);

        container.append(&info_box);

        Self {
            container,
            curve,
            drawing_area,
        }
    }

    fn create_point_row(
        index: usize,
        temp: u8,
        speed: u8,
        curve: Rc<RefCell<FanCurve>>,
        drawing_area: DrawingArea,
    ) -> adw::ActionRow {
        let row = adw::ActionRow::builder()
            .title(&format!("Point {}", index + 1))
            .build();

        // Temperature adjustment
        let temp_box = Box::new(Orientation::Horizontal, 6);
        let temp_label = Label::new(Some("Temp:"));
        temp_box.append(&temp_label);
        
        let temp_adj = gtk::Adjustment::new(temp as f64, 0.0, 100.0, 1.0, 5.0, 0.0);
        let temp_spin = gtk::SpinButton::new(Some(&temp_adj), 1.0, 0);
        temp_spin.set_width_chars(5);
        
        let curve_clone = curve.clone();
        let drawing_clone = drawing_area.clone();
        temp_spin.connect_value_changed(move |spin| {
            let mut crv = curve_clone.borrow_mut();
            if index < crv.points.len() {
                crv.points[index].0 = spin.value() as u8;
                drawing_clone.queue_draw();
            }
        });
        
        temp_box.append(&temp_spin);
        let temp_unit = Label::new(Some("°C"));
        temp_box.append(&temp_unit);
        row.add_suffix(&temp_box);

        // Speed adjustment
        let speed_box = Box::new(Orientation::Horizontal, 6);
        let speed_label = Label::new(Some("Speed:"));
        speed_box.append(&speed_label);
        
        let speed_adj = gtk::Adjustment::new(speed as f64, 0.0, 100.0, 1.0, 5.0, 0.0);
        let speed_spin = gtk::SpinButton::new(Some(&speed_adj), 1.0, 0);
        speed_spin.set_width_chars(5);
        
        let curve_clone = curve.clone();
        let drawing_clone = drawing_area.clone();
        speed_spin.connect_value_changed(move |spin| {
            let mut crv = curve_clone.borrow_mut();
            if index < crv.points.len() {
                crv.points[index].1 = spin.value() as u8;
                drawing_clone.queue_draw();
            }
        });
        
        speed_box.append(&speed_spin);
        let speed_unit = Label::new(Some("%"));
        speed_box.append(&speed_unit);
        row.add_suffix(&speed_box);

        // Delete button
        let delete_btn = Button::from_icon_name("trash-symbolic");
        delete_btn.add_css_class("destructive-action");
        delete_btn.set_valign(gtk::Align::Center);
        
        let curve_clone = curve.clone();
        let drawing_clone = drawing_area.clone();
        let row_clone = row.clone();
        delete_btn.connect_clicked(move |_| {
          let mut crv = curve_clone.borrow_mut();
          if crv.points.len() > 1 && index < crv.points.len() {
        crv.points.remove(index);
        drawing_clone.queue_draw();
        
        // Hide the row instead of removing it
        row_clone.set_visible(false);
    }
});
        
        row.add_suffix(&delete_btn);

        row
    }

    fn draw_curve(cr: &gtk::cairo::Context, width: i32, height: i32, curve: &FanCurve) {
        let w = width as f64;
        let h = height as f64;
        
        // Margins
        let margin_left = 50.0;
        let margin_right = 20.0;
        let margin_top = 20.0;
        let margin_bottom = 40.0;
        
        let chart_width = w - margin_left - margin_right;
        let chart_height = h - margin_top - margin_bottom;

        // Background
        cr.set_source_rgb(0.95, 0.95, 0.95);
        cr.paint().unwrap();

        // Draw axes
        cr.set_source_rgb(0.2, 0.2, 0.2);
        cr.set_line_width(2.0);
        
        // Y axis
        cr.move_to(margin_left, margin_top);
        cr.line_to(margin_left, h - margin_bottom);
        cr.stroke().unwrap();
        
        // X axis
        cr.move_to(margin_left, h - margin_bottom);
        cr.line_to(w - margin_right, h - margin_bottom);
        cr.stroke().unwrap();

        // Draw grid
        cr.set_source_rgb(0.8, 0.8, 0.8);
        cr.set_line_width(1.0);
        
        // Vertical grid lines (every 10°C)
        for i in 0..=10 {
            let x = margin_left + (chart_width * i as f64 / 10.0);
            cr.move_to(x, margin_top);
            cr.line_to(x, h - margin_bottom);
            cr.stroke().unwrap();
        }
        
        // Horizontal grid lines (every 10%)
        for i in 0..=10 {
            let y = margin_top + (chart_height * i as f64 / 10.0);
            cr.move_to(margin_left, y);
            cr.line_to(w - margin_right, y);
            cr.stroke().unwrap();
        }

        // Draw labels
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.set_font_size(12.0);
        
        // Y-axis label
        cr.save().unwrap();
        cr.move_to(10.0, h / 2.0);
        cr.rotate(-std::f64::consts::PI / 2.0);
        cr.show_text("Fan Speed (%)").unwrap();
        cr.restore().unwrap();
        
        // X-axis label
        cr.move_to(w / 2.0 - 50.0, h - 10.0);
        cr.show_text("Temperature (°C)").unwrap();

        // Draw curve if we have points
        if curve.points.len() >= 2 {
            let mut sorted_points = curve.points.clone();
            sorted_points.sort_by_key(|p| p.0);

            cr.set_source_rgb(0.2, 0.6, 1.0);
            cr.set_line_width(3.0);

            for (i, (temp, speed)) in sorted_points.iter().enumerate() {
                let x = margin_left + (chart_width * (*temp as f64) / 100.0);
                let y = h - margin_bottom - (chart_height * (*speed as f64) / 100.0);

                if i == 0 {
                    cr.move_to(x, y);
                } else {
                    cr.line_to(x, y);
                }
            }
            cr.stroke().unwrap();

            // Draw points
            cr.set_source_rgb(1.0, 0.3, 0.3);
            for (temp, speed) in sorted_points.iter() {
                let x = margin_left + (chart_width * (*temp as f64) / 100.0);
                let y = h - margin_bottom - (chart_height * (*speed as f64) / 100.0);
                
                cr.arc(x, y, 5.0, 0.0, 2.0 * std::f64::consts::PI);
                cr.fill().unwrap();
            }
        }
    }

    pub fn get_curve(&self) -> FanCurve {
        self.curve.borrow().clone()
    }
}
