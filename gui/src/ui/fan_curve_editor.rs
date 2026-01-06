use gtk::prelude::*;
use gtk::{Box, Button, DrawingArea, Label, Orientation, SpinButton};
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
    points_list: gtk::ListBox,
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
        drawing_area.set_content_width(600);
        drawing_area.set_content_height(400);
        drawing_area.set_vexpand(false);
        drawing_area.set_hexpand(true);
        
        let curve_clone = curve.clone();
        drawing_area.set_draw_func(move |_da, cr, width, height| {
            Self::draw_curve(cr, width, height, &curve_clone.borrow());
        });

        let frame = gtk::Frame::new(None);
        frame.set_child(Some(&drawing_area));
        container.append(&frame);

        // Points editor
        let points_group = adw::PreferencesGroup::builder()
          .title("Curve Points")
          .description("Add points to define the fan curve")
          .build();

        let points_list = gtk::ListBox::new();
        points_list.set_selection_mode(gtk::SelectionMode::None);
        points_list.add_css_class("boxed-list");

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
        let list_clone = points_list.clone();
        add_button.connect_clicked(move |_| {
            let mut crv = curve_clone.borrow_mut();
            if crv.points.len() >= 16 {
                return;
            }

            let new_temp = crv.points.last().map(|p| p.0 + 10).unwrap_or(50);
            let new_speed = 50;
            crv.points.push((new_temp, new_speed));
            drop(crv);
                
            // Rebuild all rows
            Self::rebuild_points_list(&list_clone, &curve_clone, &drawing_clone);
            drawing_clone.queue_draw();
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

        let editor = Self {
            container,
            curve: curve.clone(),
            drawing_area: drawing_area.clone(),
            points_list: points_list.clone(),
        };
        
        // Initial population of points list
        Self::rebuild_points_list(&points_list, &curve, &drawing_area);
        
        editor
    }

    fn rebuild_points_list(
        list: &gtk::ListBox,
        curve: &Rc<RefCell<FanCurve>>,
        drawing_area: &DrawingArea,
    ) {
        // Clear all existing rows
        while let Some(child) = list.first_child() {
            list.remove(&child);
        }
        
        // Recreate all rows from current curve data
        let points = curve.borrow().points.clone();
        for (index, (temp, speed)) in points.iter().enumerate() {
            let row = Self::create_point_row(
                index,
                *temp,
                *speed,
                curve.clone(),
                drawing_area.clone(),
                list.clone(),
            );
            list.append(&row);
        }
    }

    fn create_point_row(
        index: usize,
        temp: u8,
        speed: u8,
        curve: Rc<RefCell<FanCurve>>,
        drawing_area: DrawingArea,
        list: gtk::ListBox,
    ) -> adw::ActionRow {
        let row = adw::ActionRow::new();
        row.set_title(&format!("Point {}", index + 1));

        // ---- Temperature ----
        let temp_spin = SpinButton::with_range(0.0, 100.0, 1.0);
        temp_spin.set_value(temp as f64);
        temp_spin.set_valign(gtk::Align::Center);

        let curve_clone = curve.clone();
        let drawing_clone = drawing_area.clone();
        let list_clone = list.clone();
        temp_spin.connect_value_changed(move |spin| {
            let new_temp = spin.value() as u8;
            let mut crv = curve_clone.borrow_mut();
            if index < crv.points.len() {
                crv.points[index].0 = new_temp;
                drop(crv);
                drawing_clone.queue_draw();
            }
        });

        let temp_label = Label::new(Some("°C"));
        temp_label.set_margin_start(4);
        temp_label.set_valign(gtk::Align::Center);

        row.add_suffix(&temp_spin);
        row.add_suffix(&temp_label);

        // ---- Speed ----
        let speed_spin = SpinButton::with_range(0.0, 100.0, 1.0);
        speed_spin.set_value(speed as f64);
        speed_spin.set_valign(gtk::Align::Center);
        speed_spin.set_margin_start(12);

        let curve_clone = curve.clone();
        let drawing_clone = drawing_area.clone();
        speed_spin.connect_value_changed(move |spin| {
            let new_speed = spin.value() as u8;
            let mut crv = curve_clone.borrow_mut();
            if index < crv.points.len() {
                crv.points[index].1 = new_speed;
                drop(crv);
                drawing_clone.queue_draw();
            }
        });

        let speed_label = Label::new(Some("%"));
        speed_label.set_margin_start(4);
        speed_label.set_valign(gtk::Align::Center);

        row.add_suffix(&speed_spin);
        row.add_suffix(&speed_label);

        // ---- Delete ----
        let delete_btn = Button::from_icon_name("user-trash-symbolic");
        delete_btn.set_tooltip_text(Some("Delete this point"));
        delete_btn.set_valign(gtk::Align::Center);
        delete_btn.set_margin_start(12);
        delete_btn.add_css_class("flat");
        delete_btn.add_css_class("destructive-action");

        let curve_clone = curve.clone();
        let drawing_clone = drawing_area.clone();
        let list_clone = list.clone();
        delete_btn.connect_clicked(move |_| {
            let mut crv = curve_clone.borrow_mut();
            
            // Don't allow deleting if we only have 2 points (minimum requirement)
            if crv.points.len() <= 2 {
                drop(crv);
                return;
            }
            
            if index < crv.points.len() {
                crv.points.remove(index);
                drop(crv);
                
                // Rebuild entire list to update indices
                Self::rebuild_points_list(&list_clone, &curve_clone, &drawing_clone);
                drawing_clone.queue_draw();
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

        // Draw axis value labels
        for i in 0..=10 {
            // Y-axis values (0-100%)
            let y_val = 100 - i * 10;
            let y = margin_top + (chart_height * i as f64 / 10.0);
            let s = format!("{}%", y_val);
            cr.move_to(margin_left - 30.0, y + 4.0);
            cr.show_text(&s).unwrap();

            // X-axis values (0-100°C)
            let x_val = i * 10;
            let x = margin_left + (chart_width * i as f64 / 10.0);
            let s = format!("{}°", x_val);
            cr.move_to(x - 10.0, h - margin_bottom + 20.0);
            cr.show_text(&s).unwrap();
        }

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
