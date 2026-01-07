use eframe::{egui, App, Frame};
use crate::config::Config;
use crate::dbus_client::DbusClient;
use crate::egui_ui::{statistics_page, profiles_page, tuning_page, settings_page};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use tuxedo_common::types::{CpuInfo, SystemInfo, FanInfo, Profile};

enum Tab {
    Statistics,
    Profiles,
    Tuning,
    Settings,
}

pub struct TuxedoControlCenterApp {
    active_tab: Tab,
    dbus_client: Rc<RefCell<Option<DbusClient>>>,
    config: Rc<RefCell<Config>>,
    system_info: Option<SystemInfo>,
    cpu_info: Option<CpuInfo>,
    fan_info: Vec<FanInfo>,
    profiles: Vec<Profile>,
    current_profile: String,
}

impl TuxedoControlCenterApp {
    pub fn new() -> Self {
        let dbus_client = Rc::new(RefCell::new(DbusClient::new().ok()));
        let config = Rc::new(RefCell::new(Config::load().unwrap_or_default()));
        let profiles = config.borrow().data.profiles.clone();
        let current_profile = config.borrow().data.current_profile.clone();
        Self {
            active_tab: Tab::Statistics,
            dbus_client,
            config,
            system_info: None,
            cpu_info: None,
            fan_info: Vec::new(),
            profiles,
            current_profile,
        }
    }

    fn poll_data(&mut self) {
        if let Some(client) = self.dbus_client.borrow().as_ref() {
            self.system_info = client.get_system_info().ok();
            self.cpu_info = client.get_cpu_info().ok();
            self.fan_info = client.get_fan_info().unwrap_or_default();
        }
    }
}

impl Default for TuxedoControlCenterApp {
    fn default() -> Self {
        Self::new()
    }
}

impl App for TuxedoControlCenterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        self.poll_data();
        ctx.request_repaint_after(Duration::from_millis(500));

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.selectable_label(matches!(self.active_tab, Tab::Statistics), "Statistics").clicked() {
                    self.active_tab = Tab::Statistics;
                }
                if ui.selectable_label(matches!(self.active_tab, Tab::Profiles), "Profiles").clicked() {
                    self.active_tab = Tab::Profiles;
                }
                if ui.selectable_label(matches!(self.active_tab, Tab::Tuning), "Tuning").clicked() {
                    self.active_tab = Tab::Tuning;
                }
                if ui.selectable_label(matches!(self.active_tab, Tab::Settings), "Settings").clicked() {
                    self.active_tab = Tab::Settings;
                }
            });

            ui.separator();

            match self.active_tab {
                Tab::Statistics => {
                    statistics_page::statistics_page(ui, &self.system_info, &self.cpu_info, &self.fan_info);
                }
                Tab::Profiles => {
                    profiles_page::profiles_page(ui, &self.profiles, &self.current_profile, self.dbus_client.clone());
                }
                Tab::Tuning => {
                    if let Some(profile) = self.profiles.iter_mut().find(|p| p.name == self.current_profile) {
                        tuning_page::tuning_page(ui, profile, self.dbus_client.clone());
                    } else {
                        ui.label("No profile selected");
                    }
                }
                Tab::Settings => {
                    settings_page::settings_page(ui, &mut self.config.borrow_mut());
                }
            }
        });
    }
}
